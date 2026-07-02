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

use crate::bzimage::BzImageHeader;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SplitResult {
    pub header: BzImageHeader,
    pub setup_size: usize,
    pub kernel_size: usize,
}

pub fn split_bzimage(
    image_path: &Path,
    setup_path: &Path,
    kernel_path: &Path,
) -> Result<SplitResult, String> {
    let image =
        fs::read(image_path).map_err(|err| format!("read {}: {err}", image_path.display()))?;
    let header = BzImageHeader::parse(&image)?;

    // Reject images hvisor cannot load before writing output.
    crate::bzimage::report_and_validate(&header, &mut std::io::stderr())?;

    let split_at = header.protected_mode_offset;

    write_file(setup_path, &image[..split_at])?;
    write_file(kernel_path, &image[split_at..])?;

    Ok(SplitResult {
        header,
        setup_size: split_at,
        kernel_size: image.len() - split_at,
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {}: {err}", parent.display()))?;
        }
    }
    fs::write(path, bytes).map_err(|err| format!("write {}: {err}", path.display()))
}
