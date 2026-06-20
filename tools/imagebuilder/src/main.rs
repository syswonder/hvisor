// SPDX-License-Identifier: MulanPSL-2.0

mod assemble;
mod bzimage;
mod split;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Err("missing command".to_string());
    };

    match command.as_str() {
        "inspect" => {
            let image = args
                .next()
                .ok_or_else(|| "inspect requires <bzImage>".to_string())?;
            ensure_no_extra(args)?;
            inspect(Path::new(&image))
        }
        "split" => {
            let image = args
                .next()
                .ok_or_else(|| "split requires <bzImage>".to_string())?;
            let options = parse_split_options(args.collect())?;
            split_image(Path::new(&image), options)
        }
        "verify" => {
            let image = args
                .next()
                .ok_or_else(|| "verify requires <bzImage>".to_string())?;
            let options = parse_verify_options(args.collect())?;
            assemble::verify_split(Path::new(&image), &options.setup, &options.kernel)?;
            println!("PASS: split files exactly reconstruct {}", image);
            Ok(())
        }
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

struct SplitOptions {
    setup: PathBuf,
    kernel: PathBuf,
    metadata: Option<PathBuf>,
}

struct VerifyOptions {
    setup: PathBuf,
    kernel: PathBuf,
}

fn parse_split_options(args: Vec<String>) -> Result<SplitOptions, String> {
    let mut setup = None;
    let mut kernel = None;
    let mut metadata = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--setup" => setup = Some(next_path(&mut iter, "--setup")?),
            "--kernel" => kernel = Some(next_path(&mut iter, "--kernel")?),
            "--metadata" => metadata = Some(next_path(&mut iter, "--metadata")?),
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unexpected split option: {other}")),
        }
    }

    Ok(SplitOptions {
        setup: setup.ok_or_else(|| "split requires --setup <path>".to_string())?,
        kernel: kernel.ok_or_else(|| "split requires --kernel <path>".to_string())?,
        metadata,
    })
}

fn parse_verify_options(args: Vec<String>) -> Result<VerifyOptions, String> {
    let mut setup = None;
    let mut kernel = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--setup" => setup = Some(next_path(&mut iter, "--setup")?),
            "--kernel" => kernel = Some(next_path(&mut iter, "--kernel")?),
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unexpected verify option: {other}")),
        }
    }

    Ok(VerifyOptions {
        setup: setup.ok_or_else(|| "verify requires --setup <path>".to_string())?,
        kernel: kernel.ok_or_else(|| "verify requires --kernel <path>".to_string())?,
    })
}

fn next_path<I>(iter: &mut I, flag: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn ensure_no_extra<I>(mut args: I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    if let Some(extra) = args.next() {
        return Err(format!("unexpected argument: {extra}"));
    }
    Ok(())
}

fn inspect(path: &Path) -> Result<(), String> {
    let image = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let header = bzimage::BzImageHeader::parse(&image)?;

    println!("image: {}", path.display());
    println!("size_bytes: {}", image.len());
    println!("setup_sects_raw: {}", header.setup_sects_raw);
    println!("setup_sects_effective: {}", header.setup_sects);
    println!("setup_size_bytes: {}", header.protected_mode_offset);
    println!(
        "kernel_size_bytes: {}",
        image.len() - header.protected_mode_offset
    );
    println!(
        "boot_protocol_version: 0x{:04x} ({})",
        header.boot_protocol_version,
        bzimage::format_protocol_version(header.boot_protocol_version)
    );
    println!("code32_start: 0x{:08x}", header.code32_start);
    println!("loadflags: 0x{:02x}", header.loadflags);
    println!("cmd_line_ptr: 0x{:08x}", header.cmd_line_ptr);
    println!("initrd_addr_max: 0x{:08x}", header.initrd_addr_max);
    println!("kernel_alignment: 0x{:08x}", header.kernel_alignment);

    bzimage::report_and_validate(&header, &mut std::io::stdout())
}

fn split_image(image_path: &Path, options: SplitOptions) -> Result<(), String> {
    let result = split::split_bzimage(image_path, &options.setup, &options.kernel)?;
    println!(
        "setup: {} ({} bytes)",
        options.setup.display(),
        result.setup_size
    );
    println!(
        "kernel: {} ({} bytes)",
        options.kernel.display(),
        result.kernel_size
    );
    println!(
        "boot_protocol_version: 0x{:04x}",
        result.header.boot_protocol_version
    );
    println!("code32_start: 0x{:08x}", result.header.code32_start);

    if let Some(metadata_path) = options.metadata {
        write_metadata(&metadata_path, image_path, &result)?;
        println!("metadata: {}", metadata_path.display());
    }

    Ok(())
}

fn write_metadata(
    metadata_path: &Path,
    image_path: &Path,
    result: &split::SplitResult,
) -> Result<(), String> {
    if let Some(parent) = metadata_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {}: {err}", parent.display()))?;
        }
    }

    let json = format!(
        concat!(
            "{{\n",
            "  \"image\": \"{}\",\n",
            "  \"setup_sects_raw\": {},\n",
            "  \"setup_sects_effective\": {},\n",
            "  \"setup_size_bytes\": {},\n",
            "  \"kernel_size_bytes\": {},\n",
            "  \"boot_protocol_version\": \"0x{:04x}\",\n",
            "  \"code32_start\": \"0x{:08x}\"\n",
            "}}\n"
        ),
        json_escape(&image_path.display().to_string()),
        result.header.setup_sects_raw,
        result.header.setup_sects,
        result.setup_size,
        result.kernel_size,
        result.header.boot_protocol_version,
        result.header.code32_start
    );

    fs::write(metadata_path, json)
        .map_err(|err| format!("write {}: {err}", metadata_path.display()))
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

fn print_usage() {
    eprintln!(
        concat!(
            "Usage:\n",
            "  imagebuilder inspect <bzImage>\n",
            "  imagebuilder split <bzImage> --setup <setup.bin> --kernel <vmlinux.bin> [--metadata <meta.json>]\n",
            "  imagebuilder verify <bzImage> --setup <setup.bin> --kernel <vmlinux.bin>\n"
        )
    );
}
