use std::fs;
use std::path::Path;

/// Rasterize each generated SVG with resvg and confirm the resulting pixels
/// match the source PNG. The byte-level snapshot test in `compare_before_and_after.rs`
/// only catches "did the emitted SVG text change?" — this one catches
/// "are the SVG pixels still equal to the source?"
///
/// Comparison is done in **premultiplied** RGBA, which is what's actually
/// shown on screen. At low alpha, demultiplied RGB is information-poor (a
/// single unit of premultiplied rounding becomes ~255/alpha units of
/// demultiplied drift) so two visually identical pixels can have very
/// different "straight" representations. Premultiplied bytes are the visual
/// truth. We allow ±1 per channel to absorb resvg's float-math rounding.
#[test]
fn rasterized_svg_matches_source_png() {
    let images_dir = Path::new("tests/fixtures/images");
    if !images_dir.exists() {
        return;
    }

    let mut ran = false;
    for entry in fs::read_dir(images_dir).expect("Failed to read images dir") {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        ran = true;

        let source = image::open(&path)
            .unwrap_or_else(|e| panic!("Failed to open {:?}: {}", path, e))
            .into_rgba8();
        let (w, h) = (source.width(), source.height());

        let svg = pngtosvg::rgba_image_to_svg_contiguous(&source);

        let tree = usvg::Tree::from_str(&svg, &usvg::Options::default())
            .unwrap_or_else(|e| panic!("Failed to parse SVG for {:?}: {}", path, e));

        let mut pixmap = tiny_skia::Pixmap::new(w, h)
            .unwrap_or_else(|| panic!("Failed to allocate {}x{} pixmap", w, h));
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let mut mismatches: u32 = 0;
        let mut first: Option<(u32, u32, [u8; 4], [u8; 4])> = None;

        for y in 0..h {
            for x in 0..w {
                let s = source.get_pixel(x, y).0;
                // Premultiply the source straight RGBA.
                let af = s[3] as u32;
                let src_pre = [
                    ((s[0] as u32 * af + 127) / 255) as u8,
                    ((s[1] as u32 * af + 127) / 255) as u8,
                    ((s[2] as u32 * af + 127) / 255) as u8,
                    s[3],
                ];

                let pre = pixmap
                    .pixel(x, y)
                    .expect("pixmap has same dimensions as source");
                let got_pre = [pre.red(), pre.green(), pre.blue(), pre.alpha()];

                // Source pixels with alpha=0 contribute nothing visible.
                // The SVG encoder skips them entirely, so the rasterized
                // pixel must also be alpha=0 (RGB is irrelevant when alpha=0).
                if s[3] == 0 {
                    if got_pre[3] != 0 {
                        mismatches += 1;
                        first.get_or_insert((x, y, src_pre, got_pre));
                    }
                    continue;
                }

                let mut diff = false;
                for c in 0..4 {
                    if (src_pre[c] as i32 - got_pre[c] as i32).abs() > 1 {
                        diff = true;
                        break;
                    }
                }
                if diff {
                    mismatches += 1;
                    first.get_or_insert((x, y, src_pre, got_pre));
                }
            }
        }

        if mismatches > 0 {
            let (x, y, src_pre, got_pre) = first.unwrap();
            panic!(
                "{:?}: {} pixel mismatches (premultiplied); first at ({},{}) source_pre={:?} got_pre={:?}",
                path, mismatches, x, y, src_pre, got_pre
            );
        }
    }

    if !ran {
        eprintln!("No PNG fixtures found under tests/fixtures/images/");
    }
}
