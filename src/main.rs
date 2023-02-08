use clap::{Parser, ValueEnum};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_filled_rect;
use imageproc::rect::Rect;
use log::info;
use rust_htslib::{bam, bam::Read, htslib};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod axis_ticks;
pub mod identity;
pub mod utils;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Color {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
}

// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[command(author, version, about="Tool to create a length-accuracy heatmap from a cram or bam file", long_about = None)]
struct Cli {
    /// cram or bam file to create plot from
    #[arg(value_parser)]
    input: String,

    /// Number of parallel decompression threads to use
    #[arg(short, long, value_parser, default_value_t = 4)]
    threads: usize,

    /// Output file name
    #[arg(short, long, value_parser, default_value_t = String::from("accuracy_heatmap.png"))]
    output: String,

    /// Color used for heatmap
    #[arg(short, long, value_enum, value_parser, default_value_t = Color::Green)]
    color: Color,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    is_file(&args.input).unwrap_or_else(|_| panic!("Input file {} is invalid", args.input));
    info!("Collected arguments");
    let histogram = create_histogram(&args.input, args.threads);
    plot_heatmap(&histogram, args.color, &args.output);
}

fn is_file(pathname: &str) -> Result<(), String> {
    let path = PathBuf::from(pathname);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Input file {} is invalid", path.display()))
    }
}

fn create_histogram(bam_file: &str, threads: usize) -> HashMap<(usize, usize), i32> {
    let mut histogram = HashMap::new();

    let mut bam = bam::Reader::from_path(bam_file)
        .expect("Error opening BAM/CRAM file.\nIs the input file correct?\n\n\n\n");
    bam.set_threads(threads)
        .expect("Failure setting decompression threads");
    for record in bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
        .filter(|read| read.seq_len() < utils::MAX_LENGTH)
    {
        let length = utils::transform_length(record.seq_len());
        let error = utils::transform_accuracy(identity::gap_compressed_identity(record));

        if error < utils::transform_accuracy(utils::MIN_IDENTITY) {
            let entry = histogram.entry((length, error)).or_insert(0);
            *entry += 1;
        }
    }
    info!("Constructed hashmap for histogram");
    histogram
}

fn plot_heatmap(histogram: &HashMap<(usize, usize), i32>, color: Color, output: &str) {
    // Determine the maximum binned length and accuracy
    let width = utils::transform_length(utils::MAX_LENGTH);
    let height = utils::transform_accuracy(utils::MIN_IDENTITY);
    let max_value = histogram
        .values()
        .max()
        .expect("ERROR could not get max value of histogram");
    info!(
        "Figure will be {width}x{height} with {} colored pixels",
        histogram.values().len()
    );
    let mut image = RgbImage::new(width as u32, height as u32);
    for ((length, accuracy), count) in histogram {
        let intensity = (*count as f32 / *max_value as f32 * 255.0) as u8;
        let color = match color {
            Color::Red => Rgb([intensity, 0, 0]),
            Color::Green => Rgb([0, intensity, 0]),
            Color::Blue => Rgb([0, 0, intensity]),
            Color::Purple => Rgb([intensity, 0, intensity]),
            Color::Yellow => Rgb([intensity, intensity, 0]),
        };
        image = draw_filled_rect(
            &image,
            Rect::at(*length as i32, *accuracy as i32).of_size(1, 1),
            color,
        );
    }
    image = axis_ticks::add_ticks(image);
    info!("Saving image");
    image.save(output).expect("Error while saving image");
}
