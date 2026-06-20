// SPDX-License-Identifier: MulanPSL-2.0

pub const SETUP_SECTS_OFFSET: usize = 0x1f1;
pub const BOOT_FLAG_OFFSET: usize = 0x1fe;
pub const HEADER_MAGIC_OFFSET: usize = 0x202;
pub const BOOT_PROTOCOL_OFFSET: usize = 0x206;
pub const LOADFLAGS_OFFSET: usize = 0x211;
pub const CODE32_START_OFFSET: usize = 0x214;
pub const CMDLINE_PTR_OFFSET: usize = 0x228;
pub const INITRD_ADDR_MAX_OFFSET: usize = 0x22c;
pub const KERNEL_ALIGNMENT_OFFSET: usize = 0x230;
pub const MIN_HEADER_LEN: usize = 0x238;
pub const SECTOR_SIZE: usize = 512;
pub const MIN_HVISOR_BOOT_PROTOCOL: u16 = 0x0204;
pub const HVISOR_CODE32_START: u32 = 0x0010_0000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BzImageHeader {
    pub setup_sects_raw: u8,
    pub setup_sects: u8,
    pub boot_flag: u16,
    pub header_magic: [u8; 4],
    pub boot_protocol_version: u16,
    pub loadflags: u8,
    pub code32_start: u32,
    pub cmd_line_ptr: u32,
    pub initrd_addr_max: u32,
    pub kernel_alignment: u32,
    pub protected_mode_offset: usize,
}

impl BzImageHeader {
    pub fn parse(image: &[u8]) -> Result<Self, String> {
        if image.len() < MIN_HEADER_LEN {
            return Err(format!(
                "image too small for Linux/x86 setup header: {} bytes < {} bytes",
                image.len(),
                MIN_HEADER_LEN
            ));
        }

        let setup_sects_raw = image[SETUP_SECTS_OFFSET];
        let setup_sects = if setup_sects_raw == 0 {
            4
        } else {
            setup_sects_raw
        };
        let protected_mode_offset = (setup_sects as usize + 1) * SECTOR_SIZE;
        if protected_mode_offset >= image.len() {
            return Err(format!(
                "computed protected-mode offset {} is outside image size {}",
                protected_mode_offset,
                image.len()
            ));
        }

        let header_magic = read_array_4(image, HEADER_MAGIC_OFFSET);
        if &header_magic != b"HdrS" {
            return Err(format!(
                "missing Linux boot header magic at 0x{HEADER_MAGIC_OFFSET:x}: got {:?}, expected HdrS",
                String::from_utf8_lossy(&header_magic)
            ));
        }

        Ok(Self {
            setup_sects_raw,
            setup_sects,
            boot_flag: read_u16(image, BOOT_FLAG_OFFSET),
            header_magic,
            boot_protocol_version: read_u16(image, BOOT_PROTOCOL_OFFSET),
            loadflags: image[LOADFLAGS_OFFSET],
            code32_start: read_u32(image, CODE32_START_OFFSET),
            cmd_line_ptr: read_u32(image, CMDLINE_PTR_OFFSET),
            initrd_addr_max: read_u32(image, INITRD_ADDR_MAX_OFFSET),
            kernel_alignment: read_u32(image, KERNEL_ALIGNMENT_OFFSET),
            protected_mode_offset,
        })
    }

    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.setup_sects_raw == 0 {
            warnings.push("setup_sects is zero; using protocol default of 4 sectors".to_string());
        }
        if self.boot_flag != 0xaa55 {
            warnings.push(format!(
                "boot flag is 0x{:04x}, expected 0xaa55",
                self.boot_flag
            ));
        }
        if self.kernel_alignment != 0 && !self.kernel_alignment.is_power_of_two() {
            warnings.push(format!(
                "kernel_alignment 0x{:x} is not a power of two",
                self.kernel_alignment
            ));
        }
        warnings
    }

    /// Check whether hvisor can load this bzImage.
    pub fn check_loadable(&self) -> Result<(), String> {
        if self.boot_protocol_version < MIN_HVISOR_BOOT_PROTOCOL {
            return Err(format!(
                "boot protocol 0x{:04x} is older than hvisor minimum 0x{:04x}",
                self.boot_protocol_version, MIN_HVISOR_BOOT_PROTOCOL
            ));
        }
        if self.code32_start != HVISOR_CODE32_START {
            return Err(format!(
                "code32_start is 0x{:08x}, but hvisor enters the kernel at 0x{:08x}",
                self.code32_start, HVISOR_CODE32_START
            ));
        }
        Ok(())
    }
}

pub fn format_protocol_version(version: u16) -> String {
    // Linux boot protocol versions are major.minor in decimal (0x020f == 2.15).
    format!("{}.{:02}", version >> 8, version & 0xff)
}

/// Write header warnings, then enforce hvisor's load requirements.
pub fn report_and_validate<W: std::io::Write>(
    header: &BzImageHeader,
    out: &mut W,
) -> Result<(), String> {
    for warning in header.warnings() {
        writeln!(out, "WARN: {warning}").map_err(|err| format!("write warning: {err}"))?;
    }
    header.check_loadable()
}

pub fn read_u16(input: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([input[offset], input[offset + 1]])
}

pub fn read_u32(input: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ])
}

fn read_array_4(input: &[u8], offset: usize) -> [u8; 4] {
    [
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_effective_setup_size() {
        let image = fake_image(5, 0x020f, 0x0010_0000);
        let header = BzImageHeader::parse(&image).unwrap();
        assert_eq!(header.setup_sects_raw, 5);
        assert_eq!(header.setup_sects, 5);
        assert_eq!(header.protected_mode_offset, 6 * SECTOR_SIZE);
        assert_eq!(header.boot_protocol_version, 0x020f);
        assert_eq!(header.code32_start, 0x0010_0000);
    }

    #[test]
    fn applies_linux_default_when_setup_sects_is_zero() {
        let image = fake_image(0, 0x0204, 0x0010_0000);
        let header = BzImageHeader::parse(&image).unwrap();
        assert_eq!(header.setup_sects, 4);
        assert_eq!(header.protected_mode_offset, 5 * SECTOR_SIZE);
        assert!(header
            .warnings()
            .iter()
            .any(|warning| warning.contains("protocol default")));
    }

    #[test]
    fn rejects_missing_magic() {
        let mut image = fake_image(4, 0x0204, 0x0010_0000);
        image[HEADER_MAGIC_OFFSET] = b'B';
        assert!(BzImageHeader::parse(&image).is_err());
    }

    #[test]
    fn accepts_minimum_protocol() {
        let image = fake_image(5, MIN_HVISOR_BOOT_PROTOCOL, HVISOR_CODE32_START);
        let header = BzImageHeader::parse(&image).unwrap();
        assert!(header.check_loadable().is_ok());
    }

    #[test]
    fn rejects_low_protocol() {
        let image = fake_image(5, 0x0203, HVISOR_CODE32_START);
        let header = BzImageHeader::parse(&image).unwrap();
        let err = header.check_loadable().unwrap_err();
        assert!(err.contains("older than hvisor minimum"));
    }

    #[test]
    fn split_style_validation_rejects_low_protocol_image() {
        let image = fake_image(5, 0x0203, HVISOR_CODE32_START);
        let header = BzImageHeader::parse(&image).unwrap();
        let mut sink = Vec::new();
        let result = report_and_validate(&header, &mut sink);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("boot protocol 0x0203"));
    }

    #[test]
    fn validation_surfaces_warnings_then_passes_on_valid_image() {
        let mut image = fake_image(5, 0x020f, HVISOR_CODE32_START);
        image[BOOT_FLAG_OFFSET..BOOT_FLAG_OFFSET + 2].copy_from_slice(&0x1234u16.to_le_bytes());
        let header = BzImageHeader::parse(&image).unwrap();
        let mut sink = Vec::new();
        assert!(report_and_validate(&header, &mut sink).is_ok());
        let out = String::from_utf8(sink).unwrap();
        assert!(out.contains("WARN: boot flag is 0x1234"));
    }

    #[test]
    fn rejects_wrong_code32_start() {
        let image = fake_image(5, 0x020f, 0x0020_0000);
        let header = BzImageHeader::parse(&image).unwrap();
        assert!(header.check_loadable().is_err());
        let mut sink = Vec::new();
        assert!(report_and_validate(&header, &mut sink).is_err());
    }

    fn fake_image(setup_sects: u8, proto: u16, code32_start: u32) -> Vec<u8> {
        let mut image = vec![0u8; 4096];
        image[SETUP_SECTS_OFFSET] = setup_sects;
        image[BOOT_FLAG_OFFSET..BOOT_FLAG_OFFSET + 2].copy_from_slice(&0xaa55u16.to_le_bytes());
        image[HEADER_MAGIC_OFFSET..HEADER_MAGIC_OFFSET + 4].copy_from_slice(b"HdrS");
        image[BOOT_PROTOCOL_OFFSET..BOOT_PROTOCOL_OFFSET + 2].copy_from_slice(&proto.to_le_bytes());
        image[CODE32_START_OFFSET..CODE32_START_OFFSET + 4]
            .copy_from_slice(&code32_start.to_le_bytes());
        image[KERNEL_ALIGNMENT_OFFSET..KERNEL_ALIGNMENT_OFFSET + 4]
            .copy_from_slice(&0x20_0000u32.to_le_bytes());
        image
    }
}
