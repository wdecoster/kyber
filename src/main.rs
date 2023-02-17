use clap::{Parser, ValueEnum};
use image::{Rgb, RgbImage};
use log::info;
use rust_htslib::{bam, bam::Read, htslib};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod axis_ticks;
pub mod identity;
pub mod transform;

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
    /// cram or bam file, or use `-` to read from stdin
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

    /// Plot accuracy in phred scale
    #[arg(short, long, value_parser, default_value_t = false)]
    phred: bool,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    is_file(&args.input).unwrap_or_else(|_| panic!("Input file {} is invalid", args.input));
    info!("Collected arguments");
    let transform_accuracy = if args.phred {
        transform::transform_accuracy_phred
    } else {
        transform::transform_accuracy_percent
    };
    let histogram = create_histogram(&args.input, args.threads, transform_accuracy);
    plot_heatmap(
        &histogram,
        args.color,
        &args.output,
        transform_accuracy,
        args.phred,
    );
}

fn is_file(pathname: &str) -> Result<(), String> {
    if pathname == "-" {
        return Ok(());
    }
    let path = PathBuf::from(pathname);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Input file {} is invalid", path.display()))
    }
}

fn create_histogram(
    bam_file: &str,
    threads: usize,
    transform_accuracy: fn(f32) -> usize,
) -> HashMap<(usize, usize), i32> {
    let mut histogram = HashMap::new();
    let mut bam = if bam_file == "-" {
        bam::Reader::from_stdin().expect("\n\nError reading alignments from stdin.\nDid you include the file header with -h?\n\n\n\n")
    } else {
        bam::Reader::from_path(bam_file)
            .expect("Error opening BAM/CRAM file.\nIs the input file correct?\n\n\n\n")
    };
    bam.set_threads(threads)
        .expect("Failure setting decompression threads");
    for record in bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
    {
        let length = transform::transform_length(record.seq_len());
        let error = transform_accuracy(identity::gap_compressed_identity(record));

        let entry = histogram.entry((length, error)).or_insert(0);
        *entry += 1;
    }
    info!("Constructed hashmap for histogram");
    histogram
}

fn plot_heatmap(
    histogram: &HashMap<(usize, usize), i32>,
    color: Color,
    output: &str,
    transform_accuracy: fn(f32) -> usize,
    phred: bool,
) {
    let max_value = histogram
        .values()
        .max()
        .expect("ERROR could not get max value of histogram");
    info!(
        "Constructing figure with {} colored pixels",
        histogram.values().len()
    );
    let mut image = RgbImage::new(601, 601);
    for ((length, accuracy), count) in histogram {
        let intensity = (*count as f32 / *max_value as f32 * 255.0) as u8;
        let color = match color {
            Color::Red => Rgb([intensity, 0, 0]),
            Color::Green => Rgb([0, intensity, 0]),
            Color::Blue => Rgb([0, 0, intensity]),
            Color::Purple => Rgb([intensity, 0, intensity]),
            Color::Yellow => Rgb([intensity, intensity, 0]),
        };
        image.put_pixel(*length as u32, *accuracy as u32, color);
    }
    info!("Adding axis ticks");
    image = axis_ticks::add_ticks(image, transform_accuracy, phred);
    info!("Saving image");
    image.save(output).expect("Error while saving image");
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
