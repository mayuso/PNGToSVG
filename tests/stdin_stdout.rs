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

#[test]
fn file_arg_with_redirected_stdout_writes_svg_to_stdout() {
    let png_path = Path::new("tests/fixtures/images/Blue_Marble_2002_330_165.png");
    let expected = pngtosvg::convert_file_to_svg(png_path).expect("convert fixture");

    let dir = std::env::temp_dir().join(format!("pngtosvg_stdout_test_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let tmp_png = dir.join("image.png");
    fs::copy(png_path, &tmp_png).expect("copy fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg(&tmp_png)
        .stdin(Stdio::null())
        .output()
        .expect("run pngtosvg");

    let svg_written = dir.join("image.svg").exists();
    fs::remove_dir_all(&dir).ok();

    assert!(
        output.status.success(),
        "pngtosvg failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!svg_written, "no .svg file should be created");
    assert!(output.stderr.is_empty(), "stderr should be empty");
    let got = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(got.replace("\r\n", "\n"), expected.replace("\r\n", "\n"));
}

#[test]
fn invalid_file_with_redirected_stdout_fails_cleanly() {
    let dir = std::env::temp_dir().join(format!("pngtosvg_stdout_bad_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let bad = dir.join("bad.png");
    fs::write(&bad, b"not a png").expect("write bad file");

    let output = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg(&bad)
        .stdin(Stdio::null())
        .output()
        .expect("run pngtosvg");
    fs::remove_dir_all(&dir).ok();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn output_flag_writes_svg_to_given_file() {
    let png_path = Path::new("tests/fixtures/images/Blue_Marble_2002_330_165.png");
    let expected = pngtosvg::convert_file_to_svg(png_path).expect("convert fixture");

    let dir = std::env::temp_dir().join(format!("pngtosvg_output_flag_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let out_path = dir.join("custom.svg");

    let output = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg(png_path)
        .arg("-o")
        .arg(&out_path)
        .stdin(Stdio::null())
        .output()
        .expect("run pngtosvg");

    let written = fs::read_to_string(&out_path);
    fs::remove_dir_all(&dir).ok();

    assert!(
        output.status.success(),
        "pngtosvg failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "stdout should be empty");
    assert_eq!(
        written.expect("read output").replace("\r\n", "\n"),
        expected.replace("\r\n", "\n")
    );
}

#[test]
fn output_dash_writes_svg_to_stdout() {
    let png_path = Path::new("tests/fixtures/images/Blue_Marble_2002_330_165.png");
    let expected = pngtosvg::convert_file_to_svg(png_path).expect("convert fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg(png_path)
        .args(["-o", "-"])
        .stdin(Stdio::null())
        .output()
        .expect("run pngtosvg");

    assert!(output.status.success());
    let got = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(got.replace("\r\n", "\n"), expected.replace("\r\n", "\n"));
}

#[test]
fn output_flag_with_directory_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_pngtosvg"))
        .arg("tests/fixtures/images")
        .args(["-o", "x.svg"])
        .stdin(Stdio::null())
        .output()
        .expect("run pngtosvg");

    assert!(!output.status.success());
    assert!(!Path::new("x.svg").exists());
}
