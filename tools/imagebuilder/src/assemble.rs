// SPDX-License-Identifier: MulanPSL-2.0

use crate::bzimage::BzImageHeader;
use std::fs;
use std::path::Path;

pub fn verify_split(
    image_path: &Path,
    setup_path: &Path,
    kernel_path: &Path,
) -> Result<(), String> {
    let image =
        fs::read(image_path).map_err(|err| format!("read {}: {err}", image_path.display()))?;
    let header = BzImageHeader::parse(&image)?;
    // Reuse the loadability checks applied by inspect and split.
    crate::bzimage::report_and_validate(&header, &mut std::io::stderr())?;
    let setup =
        fs::read(setup_path).map_err(|err| format!("read {}: {err}", setup_path.display()))?;
    let kernel =
        fs::read(kernel_path).map_err(|err| format!("read {}: {err}", kernel_path.display()))?;

    if setup.len() != header.protected_mode_offset {
        return Err(format!(
            "setup size mismatch: got {}, expected {}",
            setup.len(),
            header.protected_mode_offset
        ));
    }

    let expected_kernel_size = image.len() - header.protected_mode_offset;
    if kernel.len() != expected_kernel_size {
        return Err(format!(
            "kernel size mismatch: got {}, expected {}",
            kernel.len(),
            expected_kernel_size
        ));
    }

    if setup.as_slice() != &image[..header.protected_mode_offset] {
        return Err("setup bytes do not match original image".to_string());
    }
    if kernel.as_slice() != &image[header.protected_mode_offset..] {
        return Err("kernel bytes do not match original image".to_string());
    }

    Ok(())
}
