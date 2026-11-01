use clap::Parser;
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use pngtosvg::{convert_file_to_svg, convert_png_bytes_to_svg};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file or directory
    #[arg(default_value = ".")]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let input_path = args.input;

    if input_path.as_os_str() == "-" {
        let mut png_bytes = Vec::new();
        io::stdin().read_to_end(&mut png_bytes)?;
        match convert_png_bytes_to_svg(&png_bytes) {
            Ok(svg) => {
                io::stdout().write_all(svg.as_bytes())?;
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to convert stdin: {e}");
                process::exit(1);
            }
        }
    }

    if !input_path.exists() {
        eprintln!("Error: Path does not exist: {}", input_path.display());
        process::exit(1);
    }

    let files_to_process: Vec<PathBuf> = if input_path.is_file() {
        if input_path.extension().and_then(|s| s.to_str()) != Some("png") {
            eprintln!("Error: The provided file is not a PNG.");
            process::exit(1);
        }
        vec![input_path]
    } else if input_path.is_dir() {
        println!("Processing directory: {}", input_path.display());
        fs::read_dir(&input_path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png")
            })
            .collect()
    } else {
        Vec::new()
    };

    let failures = files_to_process
        .par_iter()
        .filter(|path| {
            let output_path = path.with_extension("svg");

            println!("Converting {:?}...", path.file_name().unwrap_or_default());

            match convert_file_to_svg(path) {
                Ok(svg_content) => match fs::write(&output_path, svg_content) {
                    Ok(()) => {
                        println!("Success: {:?}", output_path.file_name().unwrap_or_default());
                        false
                    }
                    Err(e) => {
                        eprintln!("Failed to write {output_path:?}: {e}");
                        true
                    }
                },
                Err(e) => {
                    eprintln!("Failed to convert {path:?}: {e}");
                    true
                }
            }
        })
        .count();

    if failures > 0 {
        eprintln!(
            "{failures} of {} file(s) failed to convert.",
            files_to_process.len()
        );
        process::exit(1);
    }

    Ok(())
}
