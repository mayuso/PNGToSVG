use criterion::{Criterion, black_box, criterion_group, criterion_main};
use image::RgbaImage;
use std::path::PathBuf;

fn convert_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("svg_conversion");
    group.sample_size(100);

    let img_path =
        PathBuf::from("tests/fixtures/images/PNG_transparency_demonstration_800_600.png");

    let img = image::open(&img_path).unwrap().to_rgba8();

    group.bench_function("rgba_image_to_svg_contiguous", |b| {
        b.iter(|| {
            pngtosvg::rgba_image_to_svg_contiguous(black_box(&img));
        });
    });

    group.finish();
}

criterion_group!(benches, convert_benchmark);
criterion_main!(benches);
