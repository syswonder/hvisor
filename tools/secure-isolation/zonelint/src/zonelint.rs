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

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process;

#[derive(Debug, Clone)]
enum Json {
    Null,
    Bool,
    Number(u64),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

#[derive(Debug, Clone)]
struct ParseError {
    offset: usize,
    message: String,
}

impl ParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "json parse error at byte {}: {}",
            self.offset, self.message
        )
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<Json, ParseError> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.pos != self.bytes.len() {
            return Err(ParseError::new(self.pos, "trailing input"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<Json, ParseError> {
        self.skip_ws();
        let Some(&b) = self.bytes.get(self.pos) else {
            return Err(ParseError::new(self.pos, "unexpected end of input"));
        };
        match b {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(Json::String),
            b'0'..=b'9' => self.parse_number().map(Json::Number),
            b't' => {
                self.expect_lit(b"true")?;
                Ok(Json::Bool)
            }
            b'f' => {
                self.expect_lit(b"false")?;
                Ok(Json::Bool)
            }
            b'n' => {
                self.expect_lit(b"null")?;
                Ok(Json::Null)
            }
            _ => Err(ParseError::new(
                self.pos,
                format!("unexpected byte 0x{b:02x}"),
            )),
        }
    }

    fn parse_object(&mut self) -> Result<Json, ParseError> {
        self.expect_byte(b'{')?;
        let mut object = BTreeMap::new();
        loop {
            self.skip_ws();
            if self.consume_byte(b'}') {
                break;
            }
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            if object.insert(key.clone(), value).is_some() {
                return Err(ParseError::new(self.pos, format!("duplicate key {key:?}")));
            }
            self.skip_ws();
            if self.consume_byte(b'}') {
                break;
            }
            self.expect_byte(b',')?;
        }
        Ok(Json::Object(object))
    }

    fn parse_array(&mut self) -> Result<Json, ParseError> {
        self.expect_byte(b'[')?;
        let mut array = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_byte(b']') {
                break;
            }
            array.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_byte(b']') {
                break;
            }
            self.expect_byte(b',')?;
        }
        Ok(Json::Array(array))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        loop {
            let Some(&b) = self.bytes.get(self.pos) else {
                return Err(ParseError::new(self.pos, "unterminated string"));
            };
            self.pos += 1;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(&esc) = self.bytes.get(self.pos) else {
                        return Err(ParseError::new(self.pos, "unterminated escape"));
                    };
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            let Some(ch) = char::from_u32(code as u32) else {
                                return Err(ParseError::new(self.pos, "invalid unicode escape"));
                            };
                            out.push(ch);
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.pos - 1,
                                format!("invalid escape 0x{esc:02x}"),
                            ));
                        }
                    }
                }
                0x00..=0x1f => {
                    return Err(ParseError::new(self.pos - 1, "control character in string"));
                }
                _ => out.push(b as char),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, ParseError> {
        if self.pos + 4 > self.bytes.len() {
            return Err(ParseError::new(self.pos, "short unicode escape"));
        }
        let mut value = 0u16;
        for _ in 0..4 {
            let b = self.bytes[self.pos];
            self.pos += 1;
            value <<= 4;
            value |= match b {
                b'0'..=b'9' => (b - b'0') as u16,
                b'a'..=b'f' => (b - b'a' + 10) as u16,
                b'A'..=b'F' => (b - b'A' + 10) as u16,
                _ => return Err(ParseError::new(self.pos - 1, "invalid unicode hex digit")),
            };
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<u64, ParseError> {
        let start = self.pos;
        while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap();
        text.parse::<u64>()
            .map_err(|_| ParseError::new(start, "invalid number"))
    }

    fn expect_lit(&mut self, lit: &[u8]) -> Result<(), ParseError> {
        if self.bytes.get(self.pos..self.pos + lit.len()) == Some(lit) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err(ParseError::new(
                self.pos,
                format!("expected literal {}", String::from_utf8_lossy(lit)),
            ))
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), ParseError> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(ParseError::new(
                self.pos,
                format!("expected byte 0x{expected:02x}"),
            ))
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.bytes.get(self.pos) == Some(&expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct Region {
    kind: String,
    physical_start: u64,
    virtual_start: u64,
    size: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct VirtioTuple {
    addr: u64,
    len: u64,
    irq: u64,
}

#[derive(Debug, Clone)]
struct ZoneConfig {
    name: String,
    zone_id: u64,
    cpus: Vec<u64>,
    memory_regions: Vec<Region>,
    entry_point: u64,
    kernel_load_paddr: u64,
    arch: BTreeMap<String, Json>,
    pci_config: Vec<BTreeMap<String, Json>>,
}

#[derive(Debug, Clone)]
struct VirtioDevice {
    kind: String,
    addr: u64,
    len: u64,
    irq: u64,
    status: String,
}

#[derive(Debug, Clone)]
struct VirtioMem {
    zone0_ipa: u64,
    zonex_ipa: u64,
    size: u64,
}

#[derive(Debug, Clone)]
struct VirtioZone {
    id: u64,
    memory_regions: Vec<VirtioMem>,
    devices: Vec<VirtioDevice>,
}

#[derive(Debug, Clone)]
struct VirtioCfg {
    zones: Vec<VirtioZone>,
}

#[derive(Debug)]
struct Violation {
    rule: &'static str,
    detail: String,
}

impl Violation {
    fn new(rule: &'static str, detail: impl Into<String>) -> Self {
        Self {
            rule,
            detail: detail.into(),
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("zonelint: {err}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse()?;
    if args.zone_paths.is_empty() {
        return Err("at least one --zone <file> is required".to_string());
    }

    let mut zones = Vec::new();
    for path in &args.zone_paths {
        zones.push(load_zone(path)?);
    }

    let virtio = match &args.virtio_path {
        Some(path) => Some(load_virtio(path)?),
        None => None,
    };

    let mut violations = Vec::new();
    check_zone_ids(&zones, &mut violations);
    check_cpu_overlap(&zones, &mut violations);
    check_region_ranges(&zones, &mut violations);
    check_ram_overlap(&zones, &mut violations);
    check_mmio_exclusive(&zones, &mut violations);
    check_entry_point(&zones, &mut violations);
    check_boot_loads(&zones, &mut violations);
    check_initrd_bounds(&zones, &mut violations);
    check_pci_config(&zones, &mut violations);
    if let Some(virtio) = &virtio {
        check_virtio_consistency(&zones, virtio, &mut violations);
        check_virtio_memory_regions(virtio, &mut violations);
    }

    if violations.is_empty() {
        println!("PASS: {} zone config(s) validated", zones.len());
        if let Some(virtio) = &virtio {
            let device_count: usize = virtio.zones.iter().map(|z| z.devices.len()).sum();
            println!(
                "PASS: virtio config validated (zones={}, devices={})",
                virtio.zones.len(),
                device_count
            );
        }
        Ok(())
    } else {
        eprintln!("FAIL: {} violation(s)", violations.len());
        for v in &violations {
            eprintln!("- {}: {}", v.rule, v.detail);
        }
        process::exit(1);
    }
}

#[derive(Debug)]
struct Args {
    zone_paths: Vec<String>,
    virtio_path: Option<String>,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut zone_paths = Vec::new();
        let mut virtio_path = None;
        let mut iter = env::args().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--zone" => {
                    let Some(path) = iter.next() else {
                        return Err("--zone requires a file path".to_string());
                    };
                    zone_paths.push(path);
                }
                "--virtio" => {
                    let Some(path) = iter.next() else {
                        return Err("--virtio requires a file path".to_string());
                    };
                    virtio_path = Some(path);
                }
                "-h" | "--help" => {
                    println!(
                        "usage: zonelint --zone <zone.json> [--zone <zone.json> ...] [--virtio <virtio_cfg.json>]"
                    );
                    process::exit(0);
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }
        Ok(Self {
            zone_paths,
            virtio_path,
        })
    }
}

fn load_json(path: &str) -> Result<Json, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    Parser::new(&text)
        .parse()
        .map_err(|e| format!("{path}: {e}"))
}

fn load_zone(path: &str) -> Result<ZoneConfig, String> {
    let json = load_json(path)?;
    let root = json.as_object(path)?;
    let arch = root
        .get("arch_config")
        .ok_or_else(|| format!("{path}: missing arch_config"))?
        .as_object("arch_config")?
        .clone();
    let mut regions = Vec::new();
    for item in root
        .get("memory_regions")
        .ok_or_else(|| format!("{path}: missing memory_regions"))?
        .as_array("memory_regions")?
    {
        let object = item.as_object("memory_regions[]")?;
        regions.push(Region {
            kind: required_string(object, "type")?,
            physical_start: required_u64(object, "physical_start")?,
            virtual_start: required_u64(object, "virtual_start")?,
            size: required_u64(object, "size")?,
        });
    }
    let cpus = root
        .get("cpus")
        .ok_or_else(|| format!("{path}: missing cpus"))?
        .as_array("cpus")?
        .iter()
        .map(json_u64)
        .collect::<Result<Vec<_>, _>>()?;

    let mut pci_config = Vec::new();
    if let Some(pci) = root.get("pci_config") {
        for item in pci.as_array("pci_config")? {
            pci_config.push(item.as_object("pci_config[]")?.clone());
        }
    }

    Ok(ZoneConfig {
        name: required_string(root, "name")?,
        zone_id: required_u64(root, "zone_id")?,
        cpus,
        memory_regions: regions,
        entry_point: required_u64(root, "entry_point")?,
        kernel_load_paddr: required_u64(root, "kernel_load_paddr")?,
        arch,
        pci_config,
    })
}

fn load_virtio(path: &str) -> Result<VirtioCfg, String> {
    let json = load_json(path)?;
    let root = json.as_object(path)?;
    let mut zones = Vec::new();
    for zone in root
        .get("zones")
        .ok_or_else(|| format!("{path}: missing zones"))?
        .as_array("zones")?
    {
        let z = zone.as_object("zones[]")?;
        let mut memory_regions = Vec::new();
        for mem in z
            .get("memory_region")
            .ok_or_else(|| "virtio zone missing memory_region".to_string())?
            .as_array("memory_region")?
        {
            let m = mem.as_object("memory_region[]")?;
            memory_regions.push(VirtioMem {
                zone0_ipa: required_u64(m, "zone0_ipa")?,
                zonex_ipa: required_u64(m, "zonex_ipa")?,
                size: required_u64(m, "size")?,
            });
        }
        let mut devices = Vec::new();
        for dev in z
            .get("devices")
            .ok_or_else(|| "virtio zone missing devices".to_string())?
            .as_array("devices")?
        {
            let d = dev.as_object("devices[]")?;
            devices.push(VirtioDevice {
                kind: required_string(d, "type")?,
                addr: required_u64(d, "addr")?,
                len: required_u64(d, "len")?,
                irq: required_u64(d, "irq")?,
                status: optional_string(d, "status").unwrap_or_else(|| "enable".to_string()),
            });
        }
        zones.push(VirtioZone {
            id: required_u64(z, "id")?,
            memory_regions,
            devices,
        });
    }
    Ok(VirtioCfg { zones })
}

fn check_zone_ids(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    let mut seen = BTreeMap::new();
    for zone in zones {
        if let Some(prev) = seen.insert(zone.zone_id, zone.name.clone()) {
            violations.push(Violation::new(
                "zone-id-unique",
                format!(
                    "zone id {} reused by {} and {}",
                    zone.zone_id, prev, zone.name
                ),
            ));
        }
    }
}

fn check_cpu_overlap(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    let mut owners: BTreeMap<u64, &str> = BTreeMap::new();
    for zone in zones {
        let mut local = BTreeSet::new();
        for &cpu in &zone.cpus {
            if !local.insert(cpu) {
                violations.push(Violation::new(
                    "cpu-unique-within-zone",
                    format!("zone {} repeats cpu {}", zone.name, cpu),
                ));
            }
            if let Some(prev) = owners.insert(cpu, zone.name.as_str()) {
                violations.push(Violation::new(
                    "cpu-overlap",
                    format!("cpu {} assigned to both {} and {}", cpu, prev, zone.name),
                ));
            }
        }
    }
}

fn check_region_ranges(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    for zone in zones {
        for (idx, r) in zone.memory_regions.iter().enumerate() {
            if r.size == 0 {
                violations.push(Violation::new(
                    "region-nonzero-size",
                    format!("zone {} region {} has zero size", zone.name, idx),
                ));
            }
            if r.physical_start.checked_add(r.size).is_none() {
                violations.push(Violation::new(
                    "region-physical-overflow",
                    format!(
                        "zone {} region {} physical range overflows: start=0x{:x} size=0x{:x}",
                        zone.name, idx, r.physical_start, r.size
                    ),
                ));
            }
            if r.virtual_start.checked_add(r.size).is_none() {
                violations.push(Violation::new(
                    "region-virtual-overflow",
                    format!(
                        "zone {} region {} virtual range overflows: start=0x{:x} size=0x{:x}",
                        zone.name, idx, r.virtual_start, r.size
                    ),
                ));
            }
        }
    }
}

fn check_ram_overlap(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    let mut ranges = Vec::new();
    for zone in zones {
        for r in zone.memory_regions.iter().filter(|r| r.kind == "ram") {
            if let Some(end) = r.physical_start.checked_add(r.size) {
                ranges.push((r.physical_start, end, zone.name.as_str()));
            }
        }
    }
    ranges.sort_by_key(|r| (r.0, r.1));
    for pair in ranges.windows(2) {
        let (a_start, a_end, a_zone) = pair[0];
        let (b_start, b_end, b_zone) = pair[1];
        if a_zone != b_zone && a_end > b_start {
            violations.push(Violation::new(
                "ram-overlap",
                format!(
                    "{} RAM [0x{:x},0x{:x}) overlaps {} RAM [0x{:x},0x{:x})",
                    a_zone, a_start, a_end, b_zone, b_start, b_end
                ),
            ));
        }
    }
}

fn check_mmio_exclusive(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    let mut ranges = Vec::new();
    for zone in zones {
        for r in zone
            .memory_regions
            .iter()
            .filter(|r| matches!(r.kind.as_str(), "io" | "virtio"))
        {
            if let Some(end) = r.physical_start.checked_add(r.size) {
                ranges.push((r.physical_start, end, zone.name.as_str(), r.kind.as_str()));
            }
        }
    }
    ranges.sort_by_key(|r| (r.0, r.1));
    for pair in ranges.windows(2) {
        let (a_start, a_end, a_zone, a_kind) = pair[0];
        let (b_start, b_end, b_zone, b_kind) = pair[1];
        if a_zone != b_zone && a_end > b_start {
            violations.push(Violation::new(
                "mmio-overlap",
                format!(
                    "{} {} [0x{:x},0x{:x}) overlaps {} {} [0x{:x},0x{:x})",
                    a_zone, a_kind, a_start, a_end, b_zone, b_kind, b_start, b_end
                ),
            ));
        }
    }
}

fn check_entry_point(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    for zone in zones {
        if !contains_virtual(&zone.memory_regions, "ram", zone.entry_point, 1) {
            violations.push(Violation::new(
                "entry-point-in-ram",
                format!(
                    "zone {} entry_point=0x{:x} is not inside a RAM GPA range",
                    zone.name, zone.entry_point
                ),
            ));
        }
    }
}

fn check_boot_loads(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    for zone in zones {
        if !contains_physical(&zone.memory_regions, "ram", zone.kernel_load_paddr, 1) {
            violations.push(Violation::new(
                "kernel-load-in-ram",
                format!(
                    "zone {} kernel_load_paddr=0x{:x} is not inside declared RAM HPA",
                    zone.name, zone.kernel_load_paddr
                ),
            ));
        }

        let arch_checks = [
            ("boot_load_paddr", true),
            ("cmdline_load_hpa", true),
            ("setup_load_hpa", true),
            ("cmdline_load_gpa", false),
            ("setup_load_gpa", false),
            ("kernel_entry_gpa", false),
        ];

        for (key, physical) in arch_checks {
            let Some(value) = optional_u64(&zone.arch, key) else {
                continue;
            };
            let ok = if physical {
                contains_physical(&zone.memory_regions, "ram", value, 1)
            } else {
                contains_virtual(&zone.memory_regions, "ram", value, 1)
            };
            if !ok {
                violations.push(Violation::new(
                    "boot-address-in-ram",
                    format!(
                        "zone {} arch_config.{}=0x{:x} is outside declared RAM {}",
                        zone.name,
                        key,
                        value,
                        if physical { "HPA" } else { "GPA" }
                    ),
                ));
            }
        }
    }
}

fn check_initrd_bounds(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    for zone in zones {
        let initrd_gpa = optional_u64(&zone.arch, "initrd_load_gpa")
            .or_else(|| optional_u64(&zone.arch, "initrd_load_paddr"));
        let initrd_size = optional_u64(&zone.arch, "initrd_size").unwrap_or(0);
        if let Some(gpa) = initrd_gpa {
            if gpa != 0
                && initrd_size != 0
                && !contains_virtual(&zone.memory_regions, "reserved", gpa, initrd_size)
            {
                violations.push(Violation::new(
                    "initrd-in-reserved",
                    format!(
                        "zone {} initrd GPA [0x{:x},0x{:x}) is not inside a reserved region",
                        zone.name,
                        gpa,
                        gpa.saturating_add(initrd_size)
                    ),
                ));
            }
        }
    }
}

/// Keys required for every `pci_config` entry by hvisor-tool's
/// `parse_pci_config` (tools/hvisor.c). Each is guarded by
/// `CHECK_JSON_NULL_ERR_OUT`, so a config missing any of them fails to load on
/// the real control plane even though it is otherwise valid JSON.
const PCI_CONFIG_REQUIRED_KEYS: [&str; 14] = [
    "ecam_base",
    "ecam_size",
    "io_base",
    "io_size",
    "pci_io_base",
    "mem32_base",
    "mem32_size",
    "pci_mem32_base",
    "mem64_base",
    "mem64_size",
    "pci_mem64_base",
    "bus_range_begin",
    "bus_range_end",
    "domain",
];

fn check_pci_config(zones: &[ZoneConfig], violations: &mut Vec<Violation>) {
    for zone in zones {
        for (idx, entry) in zone.pci_config.iter().enumerate() {
            for key in PCI_CONFIG_REQUIRED_KEYS {
                if !entry.contains_key(key) {
                    violations.push(Violation::new(
                        "pci-config-complete",
                        format!(
                            "zone {} pci_config[{}] is missing required key '{}' (hvisor-tool parse_pci_config requires all {} fields)",
                            zone.name,
                            idx,
                            key,
                            PCI_CONFIG_REQUIRED_KEYS.len()
                        ),
                    ));
                }
            }
        }
    }
}

fn check_virtio_consistency(
    zones: &[ZoneConfig],
    virtio: &VirtioCfg,
    violations: &mut Vec<Violation>,
) {
    let zones_by_id: BTreeMap<u64, &ZoneConfig> = zones.iter().map(|z| (z.zone_id, z)).collect();
    let virtio_by_id: BTreeMap<u64, &VirtioZone> = virtio.zones.iter().map(|z| (z.id, z)).collect();

    for zone in zones {
        let cmdline = optional_string(&zone.arch, "cmdline").unwrap_or_default();
        let cmdline_devices = parse_cmdline_virtio_devices(&cmdline);
        let Some(vzone) = virtio_by_id.get(&zone.zone_id) else {
            if !cmdline_devices.is_empty() || zone.memory_regions.iter().any(|r| r.kind == "virtio")
            {
                violations.push(Violation::new(
                    "virtio-zone-present",
                    format!(
                        "zone {} has virtio usage but no virtio_cfg entry",
                        zone.name
                    ),
                ));
            }
            continue;
        };

        let enabled: BTreeSet<VirtioTuple> = vzone
            .devices
            .iter()
            .filter(|d| d.status == "enable")
            .map(|d| VirtioTuple {
                addr: d.addr,
                len: d.len,
                irq: d.irq,
            })
            .collect();
        let cmdline_set: BTreeSet<VirtioTuple> = cmdline_devices.into_iter().collect();

        for dev in &vzone.devices {
            if dev.status != "enable" {
                continue;
            }
            if !contains_physical(&zone.memory_regions, "virtio", dev.addr, dev.len) {
                violations.push(Violation::new(
                    "virtio-device-in-zone-region",
                    format!(
                        "zone {} virtio {} addr [0x{:x},0x{:x}) is outside zone virtio memory_regions",
                        zone.name,
                        dev.kind,
                        dev.addr,
                        dev.addr.saturating_add(dev.len)
                    ),
                ));
            }
            let tuple = VirtioTuple {
                addr: dev.addr,
                len: dev.len,
                irq: dev.irq,
            };
            if !cmdline_set.contains(&tuple) {
                violations.push(Violation::new(
                    "virtio-cmdline-match",
                    format!(
                        "zone {} virtio {} tuple len=0x{:x}@addr=0x{:x}:irq={} is missing from arch_config.cmdline",
                        zone.name, dev.kind, dev.len, dev.addr, dev.irq
                    ),
                ));
            }
        }

        for tuple in &cmdline_set {
            if !enabled.contains(tuple) {
                violations.push(Violation::new(
                    "virtio-config-match",
                    format!(
                        "zone {} cmdline tuple len=0x{:x}@addr=0x{:x}:irq={} is missing from virtio_cfg",
                        zone.name, tuple.len, tuple.addr, tuple.irq
                    ),
                ));
            }
        }
    }

    for vzone in &virtio.zones {
        if !zones_by_id.contains_key(&vzone.id) {
            violations.push(Violation::new(
                "virtio-zone-known",
                format!("virtio_cfg references unknown zone id {}", vzone.id),
            ));
        }
    }
}

fn check_virtio_memory_regions(virtio: &VirtioCfg, violations: &mut Vec<Violation>) {
    let mut zone0_ranges = Vec::new();
    for zone in &virtio.zones {
        for (idx, mem) in zone.memory_regions.iter().enumerate() {
            if mem.size == 0 {
                violations.push(Violation::new(
                    "virtio-memory-nonzero",
                    format!(
                        "virtio zone {} memory_region {} has zero size",
                        zone.id, idx
                    ),
                ));
            }
            if mem.zone0_ipa.checked_add(mem.size).is_none()
                || mem.zonex_ipa.checked_add(mem.size).is_none()
            {
                violations.push(Violation::new(
                    "virtio-memory-overflow",
                    format!("virtio zone {} memory_region {} overflows", zone.id, idx),
                ));
            }
            if let Some(end) = mem.zone0_ipa.checked_add(mem.size) {
                zone0_ranges.push((mem.zone0_ipa, end, zone.id, idx));
            }
        }
    }

    zone0_ranges.sort_by_key(|r| (r.0, r.1));
    for pair in zone0_ranges.windows(2) {
        let (a_start, a_end, a_zone, a_idx) = pair[0];
        let (b_start, b_end, b_zone, b_idx) = pair[1];
        if a_end > b_start {
            violations.push(Violation::new(
                "virtio-memory-overlap",
                format!(
                    "virtio zone {} memory_region {} [0x{:x},0x{:x}) overlaps zone {} memory_region {} [0x{:x},0x{:x}) in zone0 IPA",
                    a_zone, a_idx, a_start, a_end, b_zone, b_idx, b_start, b_end
                ),
            ));
        }
    }
}

fn parse_cmdline_virtio_devices(cmdline: &str) -> Vec<VirtioTuple> {
    let mut out = Vec::new();
    for token in cmdline.split_ascii_whitespace() {
        let Some(spec) = token.strip_prefix("virtio_mmio.device=") else {
            continue;
        };
        let Some((len_text, rest)) = spec.split_once('@') else {
            continue;
        };
        let Some((addr_text, irq_text)) = rest.split_once(':') else {
            continue;
        };
        let (Ok(len), Ok(addr), Ok(irq)) = (
            parse_int_text(len_text),
            parse_int_text(addr_text),
            parse_int_text(irq_text),
        ) else {
            continue;
        };
        out.push(VirtioTuple { addr, len, irq });
    }
    out
}

fn contains_physical(regions: &[Region], kind: &str, addr: u64, len: u64) -> bool {
    contains_range(regions, kind, addr, len, |r| r.physical_start)
}

fn contains_virtual(regions: &[Region], kind: &str, addr: u64, len: u64) -> bool {
    contains_range(regions, kind, addr, len, |r| r.virtual_start)
}

fn contains_range(
    regions: &[Region],
    kind: &str,
    addr: u64,
    len: u64,
    start_of: impl Fn(&Region) -> u64,
) -> bool {
    let Some(end) = addr.checked_add(len) else {
        return false;
    };
    regions.iter().any(|r| {
        r.kind == kind
            && start_of(r) <= addr
            && start_of(r)
                .checked_add(r.size)
                .map(|region_end| end <= region_end)
                .unwrap_or(false)
    })
}

fn required_string(map: &BTreeMap<String, Json>, key: &str) -> Result<String, String> {
    map.get(key)
        .ok_or_else(|| format!("missing key {key}"))?
        .as_string(key)
        .map(|s| s.to_string())
}

fn optional_string(map: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|v| v.as_string(key).ok())
        .map(str::to_string)
}

fn required_u64(map: &BTreeMap<String, Json>, key: &str) -> Result<u64, String> {
    map.get(key)
        .ok_or_else(|| format!("missing key {key}"))
        .and_then(json_u64)
}

fn optional_u64(map: &BTreeMap<String, Json>, key: &str) -> Option<u64> {
    map.get(key).and_then(|v| json_u64(v).ok())
}

fn json_u64(value: &Json) -> Result<u64, String> {
    match value {
        Json::Number(n) => Ok(*n),
        Json::String(s) => parse_int_text(s),
        _ => Err(format!("expected integer-compatible value, got {value:?}")),
    }
}

fn parse_int_text(text: &str) -> Result<u64, String> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| format!("invalid hex integer {text:?}: {e}"))
    } else {
        text.parse::<u64>()
            .map_err(|e| format!("invalid decimal integer {text:?}: {e}"))
    }
}

impl Json {
    fn as_object(&self, context: &str) -> Result<&BTreeMap<String, Json>, String> {
        match self {
            Json::Object(v) => Ok(v),
            _ => Err(format!("{context}: expected object, got {self:?}")),
        }
    }

    fn as_array(&self, context: &str) -> Result<&[Json], String> {
        match self {
            Json::Array(v) => Ok(v),
            _ => Err(format!("{context}: expected array, got {self:?}")),
        }
    }

    fn as_string(&self, context: &str) -> Result<&str, String> {
        match self {
            Json::String(v) => Ok(v),
            _ => Err(format!("{context}: expected string, got {self:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(name: &str, id: u64, cpus: Vec<u64>, regions: Vec<Region>) -> ZoneConfig {
        ZoneConfig {
            name: name.to_string(),
            zone_id: id,
            cpus,
            memory_regions: regions,
            entry_point: 0x8000,
            kernel_load_paddr: 0x100000,
            arch: BTreeMap::new(),
            pci_config: Vec::new(),
        }
    }

    fn ram(physical_start: u64, virtual_start: u64, size: u64) -> Region {
        Region {
            kind: "ram".to_string(),
            physical_start,
            virtual_start,
            size,
        }
    }

    #[test]
    fn cmdline_virtio_parser_accepts_hex_tuples() {
        let tuples = parse_cmdline_virtio_devices("console virtio_mmio.device=0x200@0x5950f000:10");
        assert_eq!(
            tuples,
            vec![VirtioTuple {
                addr: 0x5950_f000,
                len: 0x200,
                irq: 10
            }]
        );
    }

    #[test]
    fn ram_overlap_is_reported_across_zones() {
        let zones = vec![
            zone("a", 1, vec![1], vec![ram(0x1000, 0x8000, 0x2000)]),
            zone("b", 2, vec![2], vec![ram(0x2000, 0x8000, 0x1000)]),
        ];
        let mut violations = Vec::new();
        check_ram_overlap(&zones, &mut violations);
        assert!(violations.iter().any(|v| v.rule == "ram-overlap"));
    }

    #[test]
    fn virtio_zone0_memory_overlap_is_reported() {
        let virtio = VirtioCfg {
            zones: vec![
                VirtioZone {
                    id: 1,
                    memory_regions: vec![VirtioMem {
                        zone0_ipa: 0x1000,
                        zonex_ipa: 0,
                        size: 0x1000,
                    }],
                    devices: Vec::new(),
                },
                VirtioZone {
                    id: 2,
                    memory_regions: vec![VirtioMem {
                        zone0_ipa: 0x1800,
                        zonex_ipa: 0,
                        size: 0x1000,
                    }],
                    devices: Vec::new(),
                },
            ],
        };
        let mut violations = Vec::new();
        check_virtio_memory_regions(&virtio, &mut violations);
        assert!(violations.iter().any(|v| v.rule == "virtio-memory-overlap"));
    }
}
