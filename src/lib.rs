use image::{Rgba, RgbaImage};

type Point = (i32, i32);
type Edge = (Point, Point);

/// Reads a file from `path`, converts it to an RGBA image, and then
/// converts it to an SVG string using the `rgba_image_to_svg_contiguous` function.
///
/// # Arguments
///
/// * `path` - A path to the image file to convert.
///
/// # Returns
///
/// * `Result<String, Box<dyn std::error::Error>>` - The SVG string on success, or an error on failure.
pub fn convert_file_to_svg(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    Ok(rgba_image_to_svg_contiguous(&img))
}

/// This function processes the image to find contiguous regions of the same color
/// and generates SVG paths for them.
///
/// # Arguments
///
/// * `img` - A reference to an `RgbaImage` to convert.
///
/// # Returns
///
/// * `String` - A string containing the SVG representation of the image.
pub fn rgba_image_to_svg_contiguous(img: &RgbaImage) -> String {
    let width = img.width();
    let height = img.height();

    let raw = img.as_raw();
    let get_rgba = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * width + x) * 4) as usize;
        [raw[i], raw[i + 1], raw[i + 2], raw[i + 3]]
    };

    let mut visited = vec![false; (width * height) as usize];

    let mut svg = String::with_capacity((width * height * 5) as usize);
    svg.push_str(&svg_header(width, height));

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

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if visited[idx] {
                continue;
            }

            let rgba = get_rgba(x, y);
            if rgba[3] == 0 {
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

                        if get_rgba(nx_u, ny_u) != rgba {
                            is_boundary = true;
                        } else {
                            is_boundary = false;
                            let n_idx = (ny_u * width + nx_u) as usize;
                            if !visited[n_idx] {
                                visited[n_idx] = true;
                                queue.push((nx, ny));
                            }
                        }
                    }

                    if is_boundary {
                        let start = (here.0 + start_offset.0, here.1 + start_offset.1);
                        let end = (here.0 + end_offset.0, here.1 + end_offset.1);
                        current_edges.push((start, end));
                    }
                }
            }

            // Immediately convert current_edges to shapes and push to SVG
            if current_edges.is_empty() {
                continue;
            }

            current_edges.sort_unstable();

            used.clear();
            used.resize(current_edges.len(), false);

            let opacity = rgba[3] as f32 / 255.0;
            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

            let mut has_started_path = false;

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

                        if let Ok(idx) = current_edges.binary_search(&next_edge) {
                            if !used[idx] {
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
                    }

                    if !found || piece.first() == piece.last() {
                        break;
                    }
                }

                if piece.first() == piece.last() {
                    piece.pop();
                }

                if !piece.is_empty() {
                    if !has_started_path {
                        svg.push_str(r#" <path d=""#);
                        has_started_path = true;
                    }
                    if let Some(&start) = piece.first() {
                        let _ = write!(svg, " M {},{}", start.0, start.1);
                        for point in piece.iter().skip(1) {
                            let _ = write!(svg, " L {},{}", point.0, point.1);
                        }
                        svg.push_str(" Z");
                    }
                }
            }

            if has_started_path {
                let _ = write!(
                    svg,
                    r#"" style="fill:rgb({},{},{}); fill-opacity:{}; stroke:none;" />"#,
                    rgba[0], rgba[1], rgba[2], opacity
                );
            }
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
