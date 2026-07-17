use image::RgbaImage;
use std::collections::BTreeMap;

/// Reads the image at `path` and converts it to an SVG string using
/// [`rgba_image_to_svg_contiguous`].
///
/// # Errors
///
/// Returns an error if the file cannot be opened or decoded as an image.
pub fn convert_file_to_svg(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?.into_rgba8();
    Ok(rgba_image_to_svg_contiguous(&img))
}

/// Converts an [`RgbaImage`] to an SVG string by finding contiguous regions
/// of identical RGBA color and emitting one compact SVG path per color.
pub fn rgba_image_to_svg_contiguous(img: &RgbaImage) -> String {
    let width = img.width();
    let height = img.height();

    let raw = img.as_raw();
    let get_rgba = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * width + x) * 4) as usize;
        [raw[i], raw[i + 1], raw[i + 2], raw[i + 3]]
    };

    let mut visited = vec![false; (width * height) as usize];

    let edges_offsets = [
        ((-1, 0), ((0, 0), (0, 1))),
        ((0, 1), ((0, 1), (1, 1))),
        ((1, 0), ((1, 1), (1, 0))),
        ((0, -1), ((1, 0), (0, 0))),
    ];

    use std::fmt::Write;

    let mut queue = Vec::new();
    let mut current_edges = Vec::new();
    let mut used = Vec::new();
    let mut piece = Vec::new();
    // Subpath data accumulated per unique RGBA color. BTreeMap keeps emission
    // order deterministic across runs so snapshot tests stay stable.
    let mut paths_by_color: BTreeMap<[u8; 4], String> = BTreeMap::new();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if visited[idx] {
                continue;
            }

            let colors = get_rgba(x, y);
            if colors[3] == 0 {
                visited[idx] = true;
                continue;
            }

            queue.clear();
            queue.push((x as i32, y as i32));
            visited[idx] = true;

            current_edges.clear();

            while let Some(here) = queue.pop() {
                for &(offset, (start_offset, end_offset)) in &edges_offsets {
                    let nx = here.0 + offset.0;
                    let ny = here.1 + offset.1;

                    let is_boundary;

                    if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                        is_boundary = true;
                    } else {
                        let nx_u = nx as u32;
                        let ny_u = ny as u32;

                        if get_rgba(nx_u, ny_u) == colors {
                            is_boundary = false;
                            let n_idx = (ny_u * width + nx_u) as usize;
                            if !visited[n_idx] {
                                visited[n_idx] = true;
                                queue.push((nx, ny));
                            }
                        } else {
                            is_boundary = true;
                        }
                    }

                    if is_boundary {
                        let start = (here.0 + start_offset.0, here.1 + start_offset.1);
                        let end = (here.0 + end_offset.0, here.1 + end_offset.1);
                        current_edges.push((start, end));
                    }
                }
            }

            if current_edges.is_empty() {
                continue;
            }

            current_edges.sort_unstable();

            used.clear();
            used.resize(current_edges.len(), false);

            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
            let buf = paths_by_color.entry(colors).or_default();

            for i in 0..current_edges.len() {
                if used[i] {
                    continue;
                }
                used[i] = true;
                let first_edge = current_edges[i];

                piece.clear();
                piece.push(first_edge.0);
                piece.push(first_edge.1);

                loop {
                    let last_point = *piece.last().unwrap();
                    let mut found = false;

                    for &direction in &directions {
                        let next_point = (last_point.0 + direction.0, last_point.1 + direction.1);
                        let next_edge = (last_point, next_point);

                        if let Ok(idx) = current_edges.binary_search(&next_edge)
                            && !used[idx]
                        {
                            used[idx] = true;

                            if piece.len() >= 2 {
                                let prev_direction = (
                                    piece[piece.len() - 1].0 - piece[piece.len() - 2].0,
                                    piece[piece.len() - 1].1 - piece[piece.len() - 2].1,
                                );
                                if prev_direction == direction {
                                    piece.pop();
                                }
                            }
                            piece.push(next_point);
                            found = true;
                            break;
                        }
                    }

                    if !found || piece.first() == piece.last() {
                        break;
                    }
                }

                if piece.first() == piece.last() {
                    piece.pop();
                }

                if piece.is_empty() {
                    continue;
                }

                let (sx, sy) = piece[0];
                let _ = write!(buf, "M{},{}", sx, sy);
                let mut prev = piece[0];
                for &p in &piece[1..] {
                    let dx = p.0 - prev.0;
                    let dy = p.1 - prev.1;
                    if dy == 0 {
                        let _ = write!(buf, "h{}", dx);
                    } else {
                        let _ = write!(buf, "v{}", dy);
                    }
                    prev = p;
                }
                buf.push('Z');
            }
        }
    }

    let mut svg = String::with_capacity((width * height * 3) as usize);
    svg.push_str(&svg_header(width, height));

    for (color, data) in &paths_by_color {
        let [r, g, b, a] = *color;
        if a == 255 {
            let _ = write!(
                svg,
                r##"<path fill="#{:02x}{:02x}{:02x}" d="{}"/>"##,
                r, g, b, data
            );
        } else {
            // 3 decimals is the minimum precision for a byte-exact alpha round-trip
            // (round(a/255 * 255) == a for all a in 1..=254).
            let opacity = f32::from(a) / 255.0;
            let _ = write!(
                svg,
                r##"<path fill="#{:02x}{:02x}{:02x}" fill-opacity="{:.3}" d="{}"/>"##,
                r, g, b, opacity, data
            );
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Generates the standard SVG header.
fn svg_header(width: u32, height: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="{}" height="{}"
     xmlns="http://www.w3.org/2000/svg" version="1.1">
"#,
        width, height
    )
}
