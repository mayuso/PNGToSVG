use pretty_assertions::assert_eq;
use std::env;
use std::fs;
use std::path::Path;

#[test]
fn run_before_and_after_tests() {
    let images_dir = Path::new("tests/fixtures/images");
    let expected_dir = Path::new("tests/fixtures/expected");

    if !images_dir.exists() {
        return;
    }

    let update =
        env::var("UPDATE_EXPECTED_AFTER_IMAGES").unwrap_or_else(|_| "0".to_string()) == "1";

    let mut ran_test = false;

    for entry in fs::read_dir(images_dir).expect("Failed to read images directory") {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("png") {
            ran_test = true;
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let mut expected_path = expected_dir.join(file_stem);
            expected_path.set_extension("svg");

            let result_svg = pngtosvg::convert_file_to_svg(&path)
                .unwrap_or_else(|e| panic!("Failed to convert {:?}: {}", path, e));

            if update {
                fs::write(&expected_path, &result_svg)
                    .unwrap_or_else(|e| panic!("Failed to write expected file: {}", e));
                println!("Updated expected SVG for {}", file_stem);
            } else {
                if !expected_path.exists() {
                    panic!(
                        "Expected SVG file does not exist: {:?}.\nRun with `UPDATE_EXPECTED_AFTER_IMAGES=1 cargo test` to generate it.",
                        expected_path
                    );
                }

                let expected_svg =
                    fs::read_to_string(&expected_path).expect("Failed to read expected SVG file");

                // Normalize line endings before compare if cross-platform
                let expected_svg_normalized = expected_svg.replace("\r\n", "\n");
                let result_svg_normalized = result_svg.replace("\r\n", "\n");

                assert_eq!(
                    expected_svg_normalized, result_svg_normalized,
                    "Snapshot mismatch for image: {:?}",
                    path
                );
            }
        }
    }

    if ran_test {
        println!("Before and after comparison tests passing!");
    } else {
        println!(
            "No PNG files found in tests/fixtures/images/ to run before and after comparison tests."
        );
    }
}
