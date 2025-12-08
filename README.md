# PNGToSVG

**PNGToSVG** is a high-performance tool written in Rust that converts PNG raster images into SVG vectors.

[![Crates.io](https://img.shields.io/crates/v/pngtosvg.svg)](https://crates.io/crates/pngtosvg)

> **Note:** This project has been rewritten in **Rust** for speed and portability. 
> If you are looking for the original Python prototype, please check the [`/legacy_python`](./legacy_python) directory.

## Features

- ⚡ **Rust Performance:** Significantly faster than the original Python implementation.
- 📦 **Standalone Binary:** No Python environment or dependencies required for users.
- 🖱️ **Drag & Drop:** (Windows) Simply drag images onto the executable.
- 🛠️ **CLI Support:** Scriptable and ready for your terminal.

## Installation

### Option A: Download (Recommended for Users)
1. Go to the [Releases](https://github.com/mayuso/PNGToSVG/releases) page.
2. Download the executable for your system (`.exe` for Windows).
3. Place it anywhere on your computer (and optionally add to your PATH).

### Option B: Build from Source (For Developers)
Ensure you have [Rust installed](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/mayuso/PNGToSVG.git
cd PNGToSVG
cargo build --release
```

The binary will be available in `./target/release/`.

---

## Usage

### 1. Drag & Drop (Windows)
*Easiest for single files or quick conversions.*

1. Locate your `pngtosvg.exe`.
2. Select one or more `.png` files (or a folder containing PNGs).
3. **Drag and drop** them directly onto the `pngtosvg.exe` icon.

### 2. Command Line Interface (CLI)
*Best for automation and power users.*

**Convert a single file:**
```bash
pngtosvg image.png
```

**Convert a specific folder:**
```bash
pngtosvg ./assets/icons/
```

**Convert current directory:**
```bash
pngtosvg .
```

> **Note for Linux/macOS users:**
> If you haven't added the binary to your `PATH`, you will need to prefix the command with `./` (e.g., `./pngtosvg image.png`) while inside the directory containing the executable.

### 3. Library Usage (For calling from Rust code)

Add the dependency to your project: 

```bash
cargo add pngtosvg
```

#### Conversion from file

Use this to convert your file directly from a path:

```rust
use pngtosvg::convert_file_to_svg;
use std::path::Path;

fn main() {
    let input_path = Path::new("image.png");
    match convert_file_to_svg(input_path) {
        Ok(svg_content) => {
            println!("Generated SVG:\n{}", svg_content);
            // You can write svg_content to a file here
        }
        Err(e) => eprintln!("Error converting file: {}", e),
    }
}
```

#### In-Memory Conversion

Use this if you already have an `image::RgbaImage` (e.g., from generated content or WASM):

```rust
use pngtosvg::rgba_image_to_svg_contiguous;
use image::RgbaImage;

fn main() {
    // Assume you have an RgbaImage from somewhere
    let img = RgbaImage::new(100, 100); 
    
    // Convert directly to SVG string
    let svg_content = rgba_image_to_svg_contiguous(&img);
    
    println!("{}", svg_content);
}
```

---

## Legacy Python Version
The original Python implementation is kept in this repository for educational purposes and as a reference logic implementation. It is no longer actively maintained.

To use the legacy version:
1. Navigate to the `legacy_python` folder.
2. Install dependencies: `pip install -r requirements.txt`
3. Run: `python main.py`

## Other

A dry run verifies everything compiles and packs correctly (I do this before publishing the package)
```bash
cargo publish --dry-run
```
