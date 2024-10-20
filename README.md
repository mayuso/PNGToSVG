# PNGToSVG

Convert your PNG files to SVG.

## Usage

### Binary

To use PNGToSVG you only need to download, extract the content of the zip, copy it where your PNG files are and double click on the executable.

[Download the latest version here](https://github.com/mayuso/PNGToSVG/releases)

### Download and use code

### Python

Run:

    > cd python
    > python pngtosvg.py

Create an executable:

    > pip3 install pyinstaller
    > pyinstaller --onefile pngtosvg.py

(Last tested using [Python 3.12.5](https://www.python.org/downloads/release/python-390/))

### Rust

Run:

    > cd rust
    > cargo run path/to/dir/containing/images
    OR
    > cargo run  # Defaults to the current directory if no path is specified

Create an executable:

    > cargo build --release