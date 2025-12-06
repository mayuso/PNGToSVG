# PNGToSVG

**PNGToSVG** is a high-performance tool written in Rust that converts PNG raster images into SVG vectors.

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

---

## Legacy Python Version
The original Python implementation is kept in this repository for educational purposes and as a reference logic implementation. It is no longer actively maintained.

To use the legacy version:
1. Navigate to the `legacy_python` folder.
2. Install dependencies: `pip install -r requirements.txt`
3. Run: `python main.py`