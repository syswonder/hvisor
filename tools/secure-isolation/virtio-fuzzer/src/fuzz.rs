// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
//
// VirtIO front-end fuzzer for the hvisor / hvisor-tool split-virtqueue path.
//
// The guest writes the descriptor table, available ring and used ring; the
// zone0 backend (hvisor-tool tools/virtio/virtio.c) walks them. The split-vring
// layout written here mirrors the backend's expectations exactly:
//   * Descriptor { addr:u64, len:u32, flags:u16, next:u16 } == `struct vring_desc`
//     (typedef VirtqDesc, tools/virtio/include/virtio.h).
//   * MMIO register offsets follow the VirtIO-MMIO transport the backend serves
//     via mmio_virtio_handler / the VirtioBridge trampoline.
//
// Each malformed case targets a concrete code path in process_descriptor_chain()
// / get_virt_addr() (virtio.c):
//   - circular_descriptor : desc[0].next loops -> walk loop `vq->desc_table[next]`.
//   - oversize_len        : desc[0].len = 0xffffffff -> iov_len / indirect malloc
//                           sizing.
//   - oob_addr            : desc[0].addr outside the zone -> get_virt_addr()
//                           translation.
//   - unaligned_addr      : misaligned buffer base -> descriptor2iov().
//   - corrupt_avail_idx   : avail.idx = 0xffff -> avail-ring indexing.
//
// Two modes: --dry-run prints the crafted rings (safe anywhere, used by CI);
// --execute maps the guest MMIO window via /dev/mem, resolves vring GPAs via
// /proc/self/pagemap, programs the queue and kicks it (needs guest
// root/CAP_SYS_ADMIN and runs inside the victim zone against a live device).

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::env;
use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::os::fd::AsRawFd;
use std::process;
use std::ptr::{self, NonNull};
use std::sync::atomic::{fence, Ordering};
use std::thread;
use std::time::Duration;

const DEFAULT_MMIO_BASE: u64 = 0x5950_f000;
const DEFAULT_QUEUE: u16 = 0;
const DEFAULT_DEV_MEM: &str = "/dev/mem";
const DEFAULT_POST_KICK_MS: u64 = 250;
const PAGE_SIZE: usize = 4096;
const MMIO_REGION_SIZE: usize = 0x200;
const QUEUE_SIZE: usize = 8;
const DESC_SIZE: usize = 16;
const DESC_OFF: usize = 0;
const AVAIL_OFF: usize = DESC_OFF + DESC_SIZE * QUEUE_SIZE;
const USED_OFF: usize = PAGE_SIZE;
const VRING_BYTES: usize = PAGE_SIZE * 2;
const DATA_BYTES: usize = PAGE_SIZE;

const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

const MMIO_MAGIC: usize = 0x000;
const MMIO_VERSION: usize = 0x004;
const MMIO_DEVICE_ID: usize = 0x008;
const MMIO_VENDOR_ID: usize = 0x00c;
const MMIO_HOST_FEATURES: usize = 0x010;
const MMIO_HOST_FEATURES_SEL: usize = 0x014;
const MMIO_GUEST_FEATURES: usize = 0x020;
const MMIO_GUEST_FEATURES_SEL: usize = 0x024;
const MMIO_QUEUE_SEL: usize = 0x030;
const MMIO_QUEUE_NUM_MAX: usize = 0x034;
const MMIO_QUEUE_NUM: usize = 0x038;
const MMIO_QUEUE_ALIGN: usize = 0x03c;
const MMIO_QUEUE_PFN: usize = 0x040;
const MMIO_QUEUE_READY: usize = 0x044;
const MMIO_QUEUE_NOTIFY: usize = 0x050;
const MMIO_STATUS: usize = 0x070;
const MMIO_QUEUE_DESC_LOW: usize = 0x080;
const MMIO_QUEUE_DESC_HIGH: usize = 0x084;
const MMIO_QUEUE_DRIVER_LOW: usize = 0x090;
const MMIO_QUEUE_DRIVER_HIGH: usize = 0x094;
const MMIO_QUEUE_DEVICE_LOW: usize = 0x0a0;
const MMIO_QUEUE_DEVICE_HIGH: usize = 0x0a4;

const VIRTIO_MMIO_MAGIC: u32 = 0x7472_6976;
const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const MAP_SHARED: i32 = 0x01;

extern "C" {
    fn mmap(
        addr: *mut c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, length: usize) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[derive(Debug, Clone)]
struct AvailRing {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

impl Default for AvailRing {
    fn default() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [0; QUEUE_SIZE],
        }
    }
}

#[derive(Debug, Clone)]
struct UsedRing {
    flags: u16,
    idx: u16,
    ring: [(u32, u32); QUEUE_SIZE],
}

impl Default for UsedRing {
    fn default() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [(0, 0); QUEUE_SIZE],
        }
    }
}

#[derive(Debug, Clone)]
struct VirtqueueRing {
    desc: [Descriptor; QUEUE_SIZE],
    avail: AvailRing,
    used: UsedRing,
}

impl VirtqueueRing {
    fn new(buffer_gpa: u64) -> Self {
        let mut ring = Self {
            desc: [Descriptor::default(); QUEUE_SIZE],
            avail: AvailRing::default(),
            used: UsedRing::default(),
        };
        ring.desc[0] = Descriptor {
            addr: buffer_gpa,
            len: 512,
            flags: 0,
            next: 0,
        };
        ring.avail.ring[0] = 0;
        ring.avail.idx = 1;
        ring
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FuzzCase {
    CircularDescriptor,
    OversizeLen,
    OutOfBoundsAddr,
    UnalignedAddr,
    CorruptAvailIdx,
}

impl FuzzCase {
    fn all() -> [Self; 5] {
        [
            Self::CircularDescriptor,
            Self::OversizeLen,
            Self::OutOfBoundsAddr,
            Self::UnalignedAddr,
            Self::CorruptAvailIdx,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::CircularDescriptor => "circular_descriptor",
            Self::OversizeLen => "oversize_len",
            Self::OutOfBoundsAddr => "oob_addr",
            Self::UnalignedAddr => "unaligned_addr",
            Self::CorruptAvailIdx => "corrupt_avail_idx",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::CircularDescriptor => "desc[0] points back to itself with VRING_DESC_F_NEXT",
            Self::OversizeLen => "desc[0].len is 0xffff_ffff",
            Self::OutOfBoundsAddr => "desc[0].addr is outside the declared zone memory",
            Self::UnalignedAddr => "desc[0].addr is deliberately unaligned",
            Self::CorruptAvailIdx => "avail.idx jumps far beyond the queue size",
        }
    }
}

#[derive(Debug)]
struct Args {
    mmio_base: u64,
    queue: u16,
    dry_run: bool,
    dev_mem: String,
    post_kick_ms: u64,
    cases: Vec<FuzzCase>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("virtio-fuzzer: {err}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse()?;
    println!(
        "virtio-fuzzer: mmio_base=0x{:x} queue={} mode={} dev_mem={}",
        args.mmio_base,
        args.queue,
        if args.dry_run { "dry-run" } else { "execute" },
        args.dev_mem
    );
    if !args.dry_run {
        println!(
            "execute mode maps guest MMIO via {} and resolves vring GPAs via /proc/self/pagemap",
            args.dev_mem
        );
    }

    let mut fuzzer = VirtioFuzzer::new(
        args.mmio_base,
        args.queue,
        args.dev_mem,
        args.post_kick_ms,
        args.dry_run,
    );
    for case in args.cases {
        fuzzer.run_case(case)?;
    }
    Ok(())
}

struct VirtioFuzzer {
    mmio_base: u64,
    queue: u16,
    dev_mem: String,
    post_kick_ms: u64,
    dry_run: bool,
}

impl VirtioFuzzer {
    fn new(mmio_base: u64, queue: u16, dev_mem: String, post_kick_ms: u64, dry_run: bool) -> Self {
        Self {
            mmio_base,
            queue,
            dev_mem,
            post_kick_ms,
            dry_run,
        }
    }

    fn run_case(&mut self, case: FuzzCase) -> Result<(), String> {
        if self.dry_run {
            let mut ring = VirtqueueRing::new(0x100000);
            mutate(case, &mut ring);
            print_case(case, &ring);
            println!(
                "[{}] dry-run: would configure split vring desc=0x100000 avail=0x100080 used=0x101000 and notify queue {}",
                case.name(),
                self.queue
            );
            return Ok(());
        }

        self.execute_case(case)?;
        Ok(())
    }

    fn execute_case(&self, case: FuzzCase) -> Result<(), String> {
        if self.mmio_base == 0 {
            return Err("--mmio-base must be nonzero in execute mode".to_string());
        }

        let vring_mem = AlignedBuffer::new(VRING_BYTES, PAGE_SIZE)?;
        let data_mem = AlignedBuffer::new(DATA_BYTES, PAGE_SIZE)?;
        data_mem.fill_pattern();

        let desc_gpa = guest_phys_addr(vring_mem.as_ptr().wrapping_add(DESC_OFF))?;
        let avail_gpa = guest_phys_addr(vring_mem.as_ptr().wrapping_add(AVAIL_OFF))?;
        let used_gpa = guest_phys_addr(vring_mem.as_ptr().wrapping_add(USED_OFF))?;
        let buffer_gpa = guest_phys_addr(data_mem.as_ptr())?;

        let mut ring = VirtqueueRing::new(buffer_gpa);
        mutate(case, &mut ring);
        write_split_vring(&vring_mem, &ring);
        print_case(case, &ring);
        println!(
            "[{}] vring GPA: desc=0x{:x} avail=0x{:x} used=0x{:x} buffer=0x{:x}",
            case.name(),
            desc_gpa,
            avail_gpa,
            used_gpa,
            buffer_gpa
        );

        let mmio = MmioMap::map(&self.dev_mem, self.mmio_base, MMIO_REGION_SIZE)?;
        self.configure_queue_and_kick(&mmio, case, desc_gpa, avail_gpa, used_gpa)?;
        thread::sleep(Duration::from_millis(self.post_kick_ms));
        let used_idx =
            unsafe { ptr::read_volatile(vring_mem.as_ptr().add(USED_OFF + 2) as *const u16) };
        println!(
            "[{}] post-kick wait={}ms used.idx={}",
            case.name(),
            self.post_kick_ms,
            used_idx
        );
        Ok(())
    }

    fn configure_queue_and_kick(
        &self,
        mmio: &MmioMap,
        case: FuzzCase,
        desc_gpa: u64,
        avail_gpa: u64,
        used_gpa: u64,
    ) -> Result<(), String> {
        let magic = mmio.read32(MMIO_MAGIC);
        let version = mmio.read32(MMIO_VERSION);
        let device = mmio.read32(MMIO_DEVICE_ID);
        let vendor = mmio.read32(MMIO_VENDOR_ID);
        println!(
            "[{}] mmio identity: magic=0x{:08x} version={} device={} vendor=0x{:08x}",
            case.name(),
            magic,
            version,
            device,
            vendor
        );

        if magic != VIRTIO_MMIO_MAGIC {
            return Err(format!(
                "MMIO magic 0x{magic:08x} is not VirtIO magic 0x{VIRTIO_MMIO_MAGIC:08x}; check --mmio-base"
            ));
        }
        if device == 0 {
            return Err(
                "VirtIO device id is zero; no device present at this MMIO base".to_string(),
            );
        }
        if version == 0 {
            return Err("VirtIO MMIO version is zero".to_string());
        }

        mmio.write32(MMIO_STATUS, 0);
        mmio.write32(MMIO_STATUS, STATUS_ACKNOWLEDGE);
        mmio.write32(MMIO_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

        mmio.write32(MMIO_HOST_FEATURES_SEL, 0);
        let host_features0 = mmio.read32(MMIO_HOST_FEATURES);
        mmio.write32(MMIO_HOST_FEATURES_SEL, 1);
        let host_features1 = mmio.read32(MMIO_HOST_FEATURES);
        println!(
            "[{}] host_features[0]=0x{:08x} host_features[1]=0x{:08x}",
            case.name(),
            host_features0,
            host_features1
        );

        mmio.write32(MMIO_GUEST_FEATURES_SEL, 0);
        mmio.write32(MMIO_GUEST_FEATURES, 0);
        mmio.write32(MMIO_GUEST_FEATURES_SEL, 1);
        mmio.write32(MMIO_GUEST_FEATURES, 0);
        mmio.write32(
            MMIO_STATUS,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK,
        );

        mmio.write32(MMIO_QUEUE_SEL, self.queue as u32);
        let qmax = mmio.read32(MMIO_QUEUE_NUM_MAX);
        if qmax == 0 {
            return Err(format!(
                "queue {} is not available (QueueNumMax=0)",
                self.queue
            ));
        }
        let qsize = QUEUE_SIZE.min(qmax as usize);
        println!(
            "[{}] QueueNumMax={} configured QueueNum={}",
            case.name(),
            qmax,
            qsize
        );
        mmio.write32(MMIO_QUEUE_NUM, qsize as u32);

        if version == 1 {
            if desc_gpa % PAGE_SIZE as u64 != 0 {
                return Err(format!(
                    "legacy VirtIO requires page-aligned descriptor GPA, got 0x{desc_gpa:x}"
                ));
            }
            mmio.write32(MMIO_QUEUE_ALIGN, PAGE_SIZE as u32);
            mmio.write32(MMIO_QUEUE_PFN, (desc_gpa / PAGE_SIZE as u64) as u32);
            println!(
                "[{}] legacy queue configured align={} pfn=0x{:x}",
                case.name(),
                PAGE_SIZE,
                desc_gpa / PAGE_SIZE as u64
            );
        } else {
            write_addr(mmio, MMIO_QUEUE_DESC_LOW, MMIO_QUEUE_DESC_HIGH, desc_gpa);
            write_addr(
                mmio,
                MMIO_QUEUE_DRIVER_LOW,
                MMIO_QUEUE_DRIVER_HIGH,
                avail_gpa,
            );
            write_addr(
                mmio,
                MMIO_QUEUE_DEVICE_LOW,
                MMIO_QUEUE_DEVICE_HIGH,
                used_gpa,
            );
            mmio.write32(MMIO_QUEUE_READY, 1);
            println!("[{}] modern queue configured and marked ready", case.name());
        }

        fence(Ordering::SeqCst);
        mmio.write32(
            MMIO_STATUS,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK,
        );
        fence(Ordering::SeqCst);
        mmio.write32(MMIO_QUEUE_NOTIFY, self.queue as u32);
        println!("[{}] notified queue {}", case.name(), self.queue);
        Ok(())
    }
}

fn write_addr(mmio: &MmioMap, low_off: usize, high_off: usize, addr: u64) {
    mmio.write32(low_off, addr as u32);
    mmio.write32(high_off, (addr >> 32) as u32);
}

fn mutate(case: FuzzCase, ring: &mut VirtqueueRing) {
    match case {
        FuzzCase::CircularDescriptor => {
            ring.desc[0].flags = VRING_DESC_F_NEXT;
            ring.desc[0].next = 0;
        }
        FuzzCase::OversizeLen => {
            ring.desc[0].len = u32::MAX;
            ring.desc[0].flags = VRING_DESC_F_WRITE;
        }
        FuzzCase::OutOfBoundsAddr => {
            ring.desc[0].addr = 0xdead_beef;
            ring.desc[0].len = 4096;
        }
        FuzzCase::UnalignedAddr => {
            ring.desc[0].addr |= 1;
            ring.desc[0].len = 128;
        }
        FuzzCase::CorruptAvailIdx => {
            ring.avail.idx = u16::MAX;
            ring.avail.ring[0] = 0;
        }
    }
}

fn write_split_vring(mem: &AlignedBuffer, ring: &VirtqueueRing) {
    unsafe {
        let base = mem.as_ptr();
        for (idx, desc) in ring.desc.iter().enumerate() {
            let dst = base.add(DESC_OFF + idx * DESC_SIZE) as *mut Descriptor;
            ptr::write_volatile(dst, *desc);
        }

        write_u16(base.add(AVAIL_OFF), 0, ring.avail.flags);
        write_u16(base.add(AVAIL_OFF), 2, ring.avail.idx);
        for (idx, value) in ring.avail.ring.iter().enumerate() {
            write_u16(base.add(AVAIL_OFF), 4 + idx * 2, *value);
        }

        write_u16(base.add(USED_OFF), 0, ring.used.flags);
        write_u16(base.add(USED_OFF), 2, ring.used.idx);
        for idx in 0..QUEUE_SIZE {
            write_u32(base.add(USED_OFF), 4 + idx * 8, ring.used.ring[idx].0);
            write_u32(base.add(USED_OFF), 8 + idx * 8, ring.used.ring[idx].1);
        }
    }
    fence(Ordering::SeqCst);
}

unsafe fn write_u16(base: *mut u8, offset: usize, value: u16) {
    ptr::write_volatile(base.add(offset) as *mut u16, value);
}

unsafe fn write_u32(base: *mut u8, offset: usize, value: u32) {
    ptr::write_volatile(base.add(offset) as *mut u32, value);
}

fn print_case(case: FuzzCase, ring: &VirtqueueRing) {
    println!();
    println!("[{}] {}", case.name(), case.description());
    println!(
        "desc[0]: addr=0x{:x} len=0x{:x} flags=0x{:x} next={}",
        ring.desc[0].addr, ring.desc[0].len, ring.desc[0].flags, ring.desc[0].next
    );
    println!(
        "avail: flags=0x{:x} idx={} ring0={}",
        ring.avail.flags, ring.avail.idx, ring.avail.ring[0]
    );
    println!(
        "used: flags=0x{:x} idx={} first_id={} first_len={}",
        ring.used.flags, ring.used.idx, ring.used.ring[0].0, ring.used.ring[0].1
    );
}

struct AlignedBuffer {
    ptr: NonNull<u8>,
    layout: Layout,
    len: usize,
}

impl AlignedBuffer {
    fn new(len: usize, align: usize) -> Result<Self, String> {
        let layout = Layout::from_size_align(len, align)
            .map_err(|e| format!("invalid aligned allocation layout: {e}"))?;
        let ptr = unsafe { alloc_zeroed(layout) };
        let ptr = NonNull::new(ptr).ok_or_else(|| "aligned allocation failed".to_string())?;
        let this = Self { ptr, layout, len };
        this.fault_in_pages();
        Ok(this)
    }

    fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    fn fill_pattern(&self) {
        unsafe {
            for i in 0..self.len {
                ptr::write_volatile(self.ptr.as_ptr().add(i), (i & 0xff) as u8);
            }
        }
    }

    fn fault_in_pages(&self) {
        unsafe {
            let mut off = 0usize;
            while off < self.len {
                ptr::write_volatile(self.ptr.as_ptr().add(off), 0);
                off += PAGE_SIZE;
            }
        }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

struct MmioMap {
    _file: File,
    map_ptr: *mut u8,
    reg_ptr: *mut u8,
    map_len: usize,
}

impl MmioMap {
    fn map(path: &str, phys_addr: u64, len: usize) -> Result<Self, String> {
        let page_mask = PAGE_SIZE as u64 - 1;
        let map_base = phys_addr & !page_mask;
        let page_off = (phys_addr - map_base) as usize;
        let map_len = page_off
            .checked_add(len)
            .ok_or_else(|| "MMIO map length overflow".to_string())?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| format!("failed to open {path}: {e}"))?;

        let mapped = unsafe {
            mmap(
                ptr::null_mut(),
                map_len,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                file.as_raw_fd(),
                map_base as i64,
            )
        };
        if mapped as isize == -1 {
            return Err(format!(
                "mmap({path}, phys=0x{map_base:x}, len=0x{map_len:x}) failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let map_ptr = mapped as *mut u8;
        let reg_ptr = unsafe { map_ptr.add(page_off) };
        Ok(Self {
            _file: file,
            map_ptr,
            reg_ptr,
            map_len,
        })
    }

    fn read32(&self, offset: usize) -> u32 {
        assert!(offset + 4 <= MMIO_REGION_SIZE);
        unsafe { ptr::read_volatile(self.reg_ptr.add(offset) as *const u32) }
    }

    fn write32(&self, offset: usize, value: u32) {
        assert!(offset + 4 <= MMIO_REGION_SIZE);
        unsafe {
            ptr::write_volatile(self.reg_ptr.add(offset) as *mut u32, value);
        }
    }
}

impl Drop for MmioMap {
    fn drop(&mut self) {
        unsafe {
            munmap(self.map_ptr as *mut c_void, self.map_len);
        }
    }
}

fn guest_phys_addr(ptr: *const u8) -> Result<u64, String> {
    let vaddr = ptr as u64;
    let vpn = vaddr / PAGE_SIZE as u64;
    let offset = vaddr % PAGE_SIZE as u64;
    let mut file =
        File::open("/proc/self/pagemap").map_err(|e| format!("open /proc/self/pagemap: {e}"))?;
    file.seek(SeekFrom::Start(vpn * 8))
        .map_err(|e| format!("seek /proc/self/pagemap: {e}"))?;
    let mut bytes = [0u8; 8];
    file.read_exact(&mut bytes)
        .map_err(|e| format!("read /proc/self/pagemap: {e}"))?;
    let entry = u64::from_le_bytes(bytes);
    if entry & (1u64 << 63) == 0 {
        return Err(format!("virtual address 0x{vaddr:x} is not present"));
    }
    let pfn = entry & ((1u64 << 55) - 1);
    if pfn == 0 {
        return Err(
            "pagemap PFN is zero; run as root/CAP_SYS_ADMIN on the guest or execute in a kernel test harness"
                .to_string(),
        );
    }
    Ok(pfn * PAGE_SIZE as u64 + offset)
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut args = env::args().skip(1);
        let mut mmio_base = DEFAULT_MMIO_BASE;
        let mut queue = DEFAULT_QUEUE;
        let mut dry_run = true;
        let mut dev_mem = DEFAULT_DEV_MEM.to_string();
        let mut post_kick_ms = DEFAULT_POST_KICK_MS;
        let mut cases = Vec::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mmio-base" => {
                    let Some(value) = args.next() else {
                        return Err("--mmio-base requires a value".to_string());
                    };
                    mmio_base = parse_int(&value)?;
                }
                "--queue" => {
                    let Some(value) = args.next() else {
                        return Err("--queue requires a value".to_string());
                    };
                    queue = parse_int(&value).and_then(|v| {
                        u16::try_from(v).map_err(|_| "queue does not fit in u16".to_string())
                    })?;
                }
                "--dev-mem" => {
                    let Some(value) = args.next() else {
                        return Err("--dev-mem requires a path".to_string());
                    };
                    dev_mem = value;
                }
                "--post-kick-ms" => {
                    let Some(value) = args.next() else {
                        return Err("--post-kick-ms requires a value".to_string());
                    };
                    post_kick_ms = parse_int(&value)?;
                }
                "--case" => {
                    let Some(value) = args.next() else {
                        return Err("--case requires a value".to_string());
                    };
                    if value == "all" {
                        cases.extend(FuzzCase::all());
                    } else {
                        cases.push(parse_case(&value)?);
                    }
                }
                "--dry-run" => dry_run = true,
                "--execute" => dry_run = false,
                "-h" | "--help" => {
                    print_help();
                    process::exit(0);
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }

        if cases.is_empty() {
            cases.extend(FuzzCase::all());
        }

        Ok(Self {
            mmio_base,
            queue,
            dry_run,
            dev_mem,
            post_kick_ms,
            cases,
        })
    }
}

fn parse_case(value: &str) -> Result<FuzzCase, String> {
    match value {
        "circular_descriptor" | "circular" => Ok(FuzzCase::CircularDescriptor),
        "oversize_len" | "oversize" => Ok(FuzzCase::OversizeLen),
        "oob_addr" | "oob" => Ok(FuzzCase::OutOfBoundsAddr),
        "unaligned_addr" | "unaligned" => Ok(FuzzCase::UnalignedAddr),
        "corrupt_avail_idx" | "avail" => Ok(FuzzCase::CorruptAvailIdx),
        _ => Err(format!("unknown case: {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_fuzz_cases() {
        for case in FuzzCase::all() {
            assert_eq!(parse_case(case.name()).unwrap(), case);
        }
    }

    #[test]
    fn circular_descriptor_sets_self_referencing_next() {
        let mut ring = VirtqueueRing::new(0x100000);
        mutate(FuzzCase::CircularDescriptor, &mut ring);
        assert_eq!(ring.desc[0].flags, VRING_DESC_F_NEXT);
        assert_eq!(ring.desc[0].next, 0);
    }

    #[test]
    fn out_of_bounds_case_moves_buffer_outside_zone() {
        let mut ring = VirtqueueRing::new(0x100000);
        mutate(FuzzCase::OutOfBoundsAddr, &mut ring);
        assert_eq!(ring.desc[0].addr, 0xdead_beef);
        assert_eq!(ring.desc[0].len, 4096);
    }
}

fn parse_int(value: &str) -> Result<u64, String> {
    let value = value.trim();
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).map_err(|e| format!("invalid hex integer {value:?}: {e}"))
    } else {
        value
            .parse::<u64>()
            .map_err(|e| format!("invalid integer {value:?}: {e}"))
    }
}

fn print_help() {
    println!(
        "usage: virtio-fuzzer [--mmio-base <addr>] [--queue <n>] [--dev-mem <path>] [--post-kick-ms <ms>] [--case <name|all>] [--dry-run|--execute]"
    );
    println!(
        "cases: circular_descriptor, oversize_len, oob_addr, unaligned_addr, corrupt_avail_idx"
    );
    println!("default: --case all --dry-run --dev-mem /dev/mem --post-kick-ms 250");
    println!("execute mode requires guest root/CAP_SYS_ADMIN, /dev/mem access, and readable pagemap PFNs");
}
