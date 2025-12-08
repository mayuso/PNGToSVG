use image::{Rgba, RgbaImage};
use std::collections::{HashMap, HashSet, VecDeque};

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
    let adjacent = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut visited = vec![vec![false; img.height() as usize]; img.width() as usize];
    let mut color_pixel_lists: HashMap<Rgba<u8>, Vec<Vec<Point>>> = HashMap::new();

    for x in 0..img.width() {
        for y in 0..img.height() {
            if visited[x as usize][y as usize] {
                continue;
            }
            let rgba = img.get_pixel(x, y);
            // Skip fully transparent pixels
            if rgba[3] == 0 {
                continue;
            }
            let mut piece = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back((x as i32, y as i32));
            visited[x as usize][y as usize] = true;

            while let Some(here) = queue.pop_front() {
                for offset in &adjacent {
                    let neighbour = (here.0 + offset.0, here.1 + offset.1);
                    if neighbour.0 < 0
                        || neighbour.0 >= img.width() as i32
                        || neighbour.1 < 0
                        || neighbour.1 >= img.height() as i32
                    {
                        continue;
                    }
                    if visited[neighbour.0 as usize][neighbour.1 as usize] {
                        continue;
                    }
                    let neighbour_rgba = img.get_pixel(neighbour.0 as u32, neighbour.1 as u32);
                    if neighbour_rgba != rgba {
                        continue;
                    }
                    queue.push_back(neighbour);
                    visited[neighbour.0 as usize][neighbour.1 as usize] = true;
                }
                piece.push(here);
            }

            color_pixel_lists.entry(*rgba).or_default().push(piece);
        }
    }

    let edges = [
        ((-1, 0), ((0, 0), (0, 1))),
        ((0, 1), ((0, 1), (1, 1))),
        ((1, 0), ((1, 1), (1, 0))),
        ((0, -1), ((1, 0), (0, 0))),
    ];

    let mut color_edge_lists: HashMap<Rgba<u8>, Vec<HashSet<Edge>>> = HashMap::new();

    for (rgba, pieces) in &color_pixel_lists {
        for piece_pixel_list in pieces {
            let piece_set: HashSet<&Point> = piece_pixel_list.iter().collect();
            let mut edge_set = HashSet::new();
            for &coord in piece_pixel_list {
                for &(offset, (start_offset, end_offset)) in &edges {
                    let neighbour = (coord.0 + offset.0, coord.1 + offset.1);
                    let start = (coord.0 + start_offset.0, coord.1 + start_offset.1);
                    let end = (coord.0 + end_offset.0, coord.1 + end_offset.1);
                    let edge = (start, end);
                    if !piece_set.contains(&neighbour) {
                        edge_set.insert(edge);
                    }
                }
            }
            color_edge_lists.entry(*rgba).or_default().push(edge_set);
        }
    }

    let mut svg = String::new();
    svg.push_str(&svg_header(img.width(), img.height()));

    for (color, pieces) in &color_edge_lists {
        for edge_set in pieces {
            let shape = joined_edges(edge_set, false);
            svg.push_str(r#" <path d=""#);
            for sub_shape in shape {
                if let Some(&start) = sub_shape.first() {
                    svg.push_str(&format!(" M {},{}", start.0, start.1));
                    for &point in &sub_shape[1..] {
                        svg.push_str(&format!(" L {},{}", point.0, point.1));
                    }
                    svg.push_str(" Z");
                }
            }
            svg.push_str(&format!(
                r#"" style="fill:rgb({},{},{}); fill-opacity:{}; stroke:none;" />"#,
                color[0],
                color[1],
                color[2],
                color[3] as f32 / 255.0
            ));
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

/// Joins individual edges into a continuous path of points.
/// 
/// This is a helper for the vectorization process that traces the outline
/// of a shape from a set of edges.
fn joined_edges(assorted_edges: &HashSet<Edge>, keep_every_point: bool) -> Vec<Vec<Point>> {
    let mut pieces = Vec::new();
    let mut assorted_edges = assorted_edges.clone();
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    while !assorted_edges.is_empty() {
        let mut piece = Vec::new();
        let first_edge = assorted_edges.iter().next().unwrap().clone();
        assorted_edges.remove(&first_edge);
        piece.push(first_edge.0);
        piece.push(first_edge.1);

        loop {
            let last_point = *piece.last().unwrap();
            let mut found = false;

            for &direction in &directions {
                let next_point = (last_point.0 + direction.0, last_point.1 + direction.1);
                let next_edge = (last_point, next_point);

                if assorted_edges.contains(&next_edge) {
                    assorted_edges.remove(&next_edge);
                    if !keep_every_point && piece.len() >= 2 {
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
        pieces.push(piece);
    }

    pieces
}