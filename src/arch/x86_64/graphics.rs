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
//  Solicey <lzoi_lth@163.com>

use crate::arch::boot::get_multiboot_tags;
use spin::{Mutex, Once};

const PSF2_MAGIC: u32 = 0x864ab572;

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct Psf2Header {
    magic: u32,
    version: u32,
    header_size: u32,
    flags: u32,
    glyph_nr: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct FontInfo {
    /// width in bytes (8 pixels)
    width_bytes: usize,
    /// width in pixels
    width: usize,
    /// height in pixels
    height: usize,
    /// table address     
    glyph_table: usize,
    /// number of glyphs
    glyph_nr: u32,
    /// size of each glyph
    bytes_per_glyph: u32,
}

static FONT_INFO: Once<FontInfo> = Once::new();

#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// x in char
    cursor_x: usize,
    /// y in char
    cursor_y: usize,
    max_char_nr_x: usize,
    max_char_nr_y: usize,
    pub addr: usize,
    pub width: usize,
    pub height: usize,
}

static FRAMEBUFFER_INFO: Once<Mutex<FramebufferInfo>> = Once::new();

pub fn font_init(psf: &'static [u8]) {
    let psf_header = unsafe { *(psf.as_ptr() as *const Psf2Header) };
    // only support psf2
    assert!(psf_header.magic == PSF2_MAGIC);

    let font_width_bytes = (psf_header.width + 7) / 8; // up align to 8bit
    let font_width = font_width_bytes * 8;

    // println!("{:#x?}", psf_header);

    FONT_INFO.call_once(|| FontInfo {
        width: font_width as _,
        height: psf_header.height as _,
        glyph_table: (psf.as_ptr() as usize + psf_header.header_size as usize),
        glyph_nr: psf_header.glyph_nr,
        bytes_per_glyph: psf_header.bytes_per_glyph,
        width_bytes: font_width_bytes as _,
    });

    let framebuffer = &get_multiboot_tags().framebuffer;
    FRAMEBUFFER_INFO.call_once(|| {
        Mutex::new(FramebufferInfo {
            cursor_x: 0,
            cursor_y: 0,
            max_char_nr_x: (framebuffer.width / font_width) as _,
            max_char_nr_y: (framebuffer.height / psf_header.height) as _,
            addr: framebuffer.addr as _,
            width: framebuffer.width as _,
            height: framebuffer.height as _,
        })
    });

    fb_clear_screen();
}

fn fb_clear_screen() {
    let mut fb_info = FRAMEBUFFER_INFO.get().unwrap().lock();
    let mut ptr = fb_info.addr as *mut u32;
    for height in 0..fb_info.height {
        for width in 0..fb_info.width {
            unsafe {
                core::ptr::write_volatile(ptr, 0);
                ptr = ptr.wrapping_add(1);
            }
        }
    }
}

fn fb_putchar_internal(ch: u16, fg: u32, bg: u32) {
    let font_info = FONT_INFO.get().unwrap();
    let mut glyph = font_info.glyph_table as *const u8;

    if (ch as u32) < font_info.glyph_nr {
        glyph = glyph.wrapping_add((ch as usize) * (font_info.bytes_per_glyph as usize));
    }

    {
        let mut fb_info = FRAMEBUFFER_INFO.get().unwrap().lock();
        // current pixel
        let cur = fb_info.cursor_y * font_info.height * fb_info.width
            + fb_info.cursor_x * font_info.width;
        let base = fb_info.addr as *mut u32;

        for y in 0..font_info.height {
            let mut mask: u8 = 1 << 7;
            for x in 0..font_info.width {
                if x % 8 == 0 {
                    mask = 1 << 7;
                }

                let color = match unsafe { *glyph.wrapping_add(x / 8) } & mask != 0 {
                    true => fg,
                    false => bg,
                };

                let ptr = base.wrapping_add(cur + y * fb_info.width + x);
                unsafe { core::ptr::write_volatile(ptr, color) };

                mask = mask >> 1;
            }

            glyph = glyph.wrapping_add(font_info.width_bytes);
        }

        fb_info.cursor_x += 1;
        if fb_info.cursor_x < fb_info.max_char_nr_x {
            return;
        }
    }

    fb_putchar_new_line(bg);
}

fn fb_putchar_new_line(bg: u32) {
    let font_info = FONT_INFO.get().unwrap();
    let mut fb_info = FRAMEBUFFER_INFO.get().unwrap().lock();
    let base = fb_info.addr as *mut u32;

    fb_info.cursor_x = 0;
    fb_info.cursor_y += 1;

    if fb_info.cursor_y >= fb_info.max_char_nr_y {
        fb_info.cursor_y = 0;
    }

    for y in 0..font_info.height {
        let y1 = (y + fb_info.cursor_y * font_info.height) * fb_info.width;
        for x in 0..fb_info.width {
            unsafe { core::ptr::write_volatile(base.wrapping_add(x + y1), bg) };
        }
    }

    // may need to scroll up
    /*if fb_info.cursor_y >= fb_info.max_char_nr_y {
        for y in 0..((fb_info.max_char_nr_y - 1) * font_info.height) {
            let y1 = y * fb_info.width;
            let y2 = (y + font_info.height) * fb_info.width;
            for x in 0..fb_info.width {
                unsafe {
                    core::ptr::write_volatile(
                        base.wrapping_add(x + y1),
                        core::ptr::read_volatile(base.wrapping_add(x + y2)),
                    )
                };
            }
        }

        for y in 0..font_info.height {
            let y1 = (y + (fb_info.max_char_nr_y - 1) * font_info.height) * fb_info.width;
            for x in 0..fb_info.width {
                unsafe { core::ptr::write_volatile(base.wrapping_add(x + y1), bg) };
            }
        }

        fb_info.cursor_y -= 1;
    }*/
}

pub fn fb_putchar(ch: u8, fg: u32, bg: u32) {
    match ch as char {
        '\r' => {}
        '\n' => fb_putchar_new_line(bg),
        _ => fb_putchar_internal(ch as _, fg, bg),
    }
}

pub fn fb_putstr(s: &str, fg: u32) {
    for c in s.chars() {
        match c {
            '\n' => {
                fb_putchar_new_line(0x0);
            }
            _ => fb_putchar_internal(c as _, fg, 0x0),
        }
    }
}
