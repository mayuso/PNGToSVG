use clap::Parser;
use image::{Rgba, RgbaImage};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, process};

// Define the arguments struct
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file or directory
    #[arg(default_value = ".")]
    input: PathBuf,
}

type Point = (i32, i32);
type Edge = (Point, Point);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments using clap
    let args = Cli::parse();
    let input_path = &args.input;

    if !input_path.exists() {
        eprintln!("Error: Path does not exist: {}", input_path.display());
        process::exit(1);
    }

    // Collect list of files to process
    let files_to_process: Vec<PathBuf> = if input_path.is_file() {
        // Single file
        if input_path.extension().and_then(|s| s.to_str()) != Some("png") {
            eprintln!("Error: The provided file is not a PNG.");
            process::exit(1);
        }
        vec![input_path.to_path_buf()]
    } else if input_path.is_dir() {
        // Directory    
        println!("Processing directory: {}", input_path.display());
        fs::read_dir(input_path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png"))
            .collect()
    } else {
        Vec::new()
    };

    files_to_process.par_iter().for_each(|path| {
        let output_path = path.with_extension("svg");
        
        println!("Converting {:?}...", path.file_name().unwrap_or_default());

        match png_to_svg(path) {
            Ok(svg) => {
                if let Ok(mut file) = File::create(&output_path) {
                    let _ = file.write_all(svg.as_bytes());
                    println!("Success: {:?}", output_path.file_name().unwrap_or_default());
                }
            }
            Err(e) => eprintln!("Failed to convert {:?}: {}", path, e),
        }
    });

    Ok(())
}

fn png_to_svg(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    Ok(rgba_image_to_svg_contiguous(&img))
}

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

fn rgba_image_to_svg_contiguous(img: &RgbaImage) -> String {
    let adjacent = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut visited = vec![vec![false; img.height() as usize]; img.width() as usize];
    let mut color_pixel_lists: HashMap<Rgba<u8>, Vec<Vec<Point>>> = HashMap::new();

    for x in 0..img.width() {
        for y in 0..img.height() {
            if visited[x as usize][y as usize] {
                continue;
            }
            let rgba = img.get_pixel(x, y);
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
            let mut edge_set = HashSet::new();
            for &coord in piece_pixel_list {
                for &(offset, (start_offset, end_offset)) in &edges {
                    let neighbour = (coord.0 + offset.0, coord.1 + offset.1);
                    let start = (coord.0 + start_offset.0, coord.1 + start_offset.1);
                    let end = (coord.0 + end_offset.0, coord.1 + end_offset.1);
                    let edge = (start, end);
                    if !piece_pixel_list.contains(&neighbour) {
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