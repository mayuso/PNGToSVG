use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn stdin_dash_writes_svg_to_stdout() {
    let png_path = Path::new("tests/fixtures/images/Blue_Marble_2002_330_165.png");
    if !png_path.exists() {
        return;
    }

    let png_bytes = fs::read(png_path).expect("read fixture PNG");
    let expected = pngtosvg::convert_file_to_svg(png_path).expect("convert fixture");

    let mut child = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pngtosvg");

    {
        let stdin = child.stdin.as_mut().expect("child stdin");
        stdin.write_all(&png_bytes).expect("write PNG to stdin");
    }

    let output = child.wait_with_output().expect("wait for pngtosvg");
    assert!(
        output.status.success(),
        "pngtosvg - failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let got = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(got.replace("\r\n", "\n"), expected.replace("\r\n", "\n"));
}

#[test]
fn convert_png_bytes_matches_file_conversion() {
    let png_path = Path::new("tests/fixtures/images/Blue_Marble_2002_330_165.png");
    if !png_path.exists() {
        return;
    }

    let bytes = fs::read(png_path).expect("read fixture PNG");
    let from_bytes = pngtosvg::convert_png_bytes_to_svg(&bytes).expect("convert bytes");
    let from_file = pngtosvg::convert_file_to_svg(png_path).expect("convert file");

    assert_eq!(
        from_bytes.replace("\r\n", "\n"),
        from_file.replace("\r\n", "\n")
    );
}
