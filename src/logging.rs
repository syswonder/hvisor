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
use core::fmt::{self, Write};

use log::{self, Level, LevelFilter, Log, Metadata, Record};
use spin::Mutex;

use crate::device::uart;

static PRINT_LOCK: Mutex<()> = Mutex::new(());
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => {
                    uart::console_putchar(b'\r');
                    uart::console_putchar(b'\n');
                }
                _ => uart::console_putchar(c as u8),
            }
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    let _locked = PRINT_LOCK.lock();
    Stdout.write_fmt(args).unwrap();
}
/// print without line breaks
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! zone_error {
    ($($arg:tt)*) => {{
        error!($($arg)*);
        zone_error();
    }};
}

/// print with line breaks
#[macro_export]
macro_rules! println {
    () => { print!("\n") };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

macro_rules! with_color {
    ($color_code:expr, $($arg:tt)*) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[m", $color_code as u8, format_args!($($arg)*))
    }};
}

#[repr(u8)]
#[allow(dead_code)]
enum ColorCode {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

fn color_code_to_bgra(code: &ColorCode) -> u32 {
    match code {
        ColorCode::Black => 0,
        ColorCode::Red => 0x0000aaff,
        ColorCode::Green => 0x00aa00ff,
        ColorCode::Yellow => 0x0055aaff,
        ColorCode::Blue => 0xaa0000ff,
        ColorCode::Magenta => 0xaa00aaff,
        ColorCode::Cyan => 0xaaaa00ff,
        ColorCode::White => 0xaaaaaaff,
        ColorCode::BrightBlack => 0x555555ff,
        ColorCode::BrightRed => 0x5555ffff,
        ColorCode::BrightGreen => 0x55ff55ff,
        ColorCode::BrightYellow => 0x55ffffff,
        ColorCode::BrightBlue => 0xff5555ff,
        ColorCode::BrightMagenta => 0xff55ffff,
        ColorCode::BrightCyan => 0xffff55ff,
        ColorCode::BrightWhite => 0xffffffff,
        _ => 0,
    }
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

struct SimpleLogger;

impl SimpleLogger {
    #[cfg(feature = "graphics")]
    fn print(
        &self,
        level: Level,
        line: u32,
        target: &str,
        cpu_id: usize,
        level_color: ColorCode,
        args_color: ColorCode,
        record: &Record,
    ) {
        println!(
            "[{:<5} {}] ({}:{}) {}",
            level,
            cpu_id,
            target,
            line,
            record.args()
        );
    }

    #[cfg(not(feature = "graphics"))]
    fn print(
        &self,
        level: Level,
        line: u32,
        target: &str,
        cpu_id: usize,
        level_color: ColorCode,
        args_color: ColorCode,
        record: &Record,
    ) {
        #[cfg(feature = "print_timestamp")]
        {
            let time_us: u64 = crate::arch::time::get_time_us();
            let sec = time_us / 1_000_000;
            let us = time_us % 1_000_000;
            print(with_color!(
                ColorCode::White,
                "[{}] {} {} hvisor: {} {}\n",
                with_color!(ColorCode::BrightWhite, "{:>5}.{:06}", sec, us),
                with_color!(level_color, "{:<5}", level),
                with_color!(ColorCode::BrightGreen, "CPU{}", cpu_id),
                with_color!(ColorCode::White, "({}:{})", target, line),
                with_color!(args_color, "{}", record.args()),
            ));
        }
        #[cfg(not(feature = "print_timestamp"))]
        print(with_color!(
            ColorCode::White,
            "[{} {}] {} {}\n",
            with_color!(level_color, "{:<5}", level),
            with_color!(ColorCode::White, "{}", cpu_id),
            with_color!(ColorCode::White, "({}:{})", target, line),
            with_color!(args_color, "{}", record.args()),
        ));
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let line = record.line().unwrap_or(0);
        let target = record.target();
        let cpu_id = crate::cpu_data::this_cpu_data().id;
        let level_color = match level {
            Level::Error => ColorCode::BrightRed,
            Level::Warn => ColorCode::BrightYellow,
            Level::Info => ColorCode::BrightGreen,
            Level::Debug => ColorCode::BrightCyan,
            Level::Trace => ColorCode::BrightBlack,
        };
        let args_color = match level {
            Level::Error => ColorCode::Red,
            Level::Warn => ColorCode::Yellow,
            Level::Info => ColorCode::Green,
            Level::Debug => ColorCode::Cyan,
            Level::Trace => ColorCode::BrightBlack,
        };

        self.print(level, line, target, cpu_id, level_color, args_color, record);
    }

    fn flush(&self) {}
}
