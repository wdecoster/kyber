use clap::Parser;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_filled_rect;
use imageproc::rect::Rect;
use log::info;
use rust_htslib::{bam, bam::Read, htslib};
use std::collections::HashMap;
use std::path::PathBuf;

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
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    is_file(&args.input).unwrap_or_else(|_| panic!("Input file {} is invalid", args.input));
    info!("Collected arguments");
    let matrix = create_histogram(&args.input, args.threads);
    plot_heatmap(&matrix, &args.output);
}

fn is_file(pathname: &str) -> Result<(), String> {
    let path = PathBuf::from(pathname);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Input file {} is invalid", path.display()))
    }
}

fn create_histogram(bam_file: &str, threads: usize) -> Vec<Vec<usize>> {
    let length_bin_size = 500;
    let mut histogram = HashMap::new();

    let mut bam = bam::Reader::from_path(bam_file)
        .expect("Error opening BAM/CRAM file.\nIs the input file correct?\n\n\n\n");
    bam.set_threads(threads)
        .expect("Failure setting decompression threads");
    for record in bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
    {
        let length = record.seq().len();
        let accuracy = record.mapq() as usize;
        let binned_length = (length / length_bin_size) * length_bin_size;

        // Increment the count for the given binned length and accuracy in the histogram
        let entry = histogram.entry((binned_length, accuracy)).or_insert(0);
        *entry += 1;
    }

    // Determine the maximum binned length and accuracy
    let max_binned_length = histogram.keys().map(|(length, _)| length).max().unwrap();
    let max_accuracy = histogram
        .keys()
        .map(|(_, accuracy)| accuracy)
        .max()
        .unwrap();

    // Initialize the matrix with zeros
    let mut matrix = vec![vec![0; max_accuracy + 1]; max_binned_length + 1];

    // Fill in the matrix with the counts from the histogram
    for ((binned_length, accuracy), count) in histogram {
        matrix[binned_length][accuracy] = count;
    }

    matrix
}

fn plot_heatmap(matrix: &Vec<Vec<usize>>, output: &str) {
    let width = matrix.len();
    let height = matrix[0].len();
    let max_value = matrix
        .iter()
        .flat_map(|row| row.iter())
        .cloned()
        .max()
        .unwrap();

    let mut image = RgbImage::new(width as u32, height as u32);

    for (x, row) in matrix.iter().enumerate() {
        for (y, &value) in row.iter().enumerate() {
            let color = Rgb([(value as f32 / max_value as f32 * 255.0) as u8, 0, 0]);
            image = draw_filled_rect(&image, Rect::at(x as i32, y as i32).of_size(1, 1), color);
        }
    }

    image.save(output).unwrap();
}
