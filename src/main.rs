use clap::{Parser, ValueEnum};
use image::{Rgb, RgbImage};
use log::info;
use ndarray::{arr1, Array1};
use std::collections::HashMap;

pub mod axis_ticks;
pub mod extract_data;
pub mod identity;
pub mod transform;
pub mod utils;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Color {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum BackGround {
    Black,
    White,
}

// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[command(author, version, about="Tool to create a length-accuracy heatmap from a cram or bam file", long_about = None)]
struct Cli {
    /// cram or bam file(s), or use `-` to read a file from stdin with e.g. samtools view -h
    #[arg(short, long, value_parser, num_args = 0..=3, required = true)]
    input: Vec<String>,

    /// Number of parallel decompression threads to use
    #[arg(short, long, value_parser, default_value_t = 4)]
    threads: usize,

    /// Output file name
    #[arg(short, long, value_parser, default_value_t = String::from("accuracy_heatmap.png"))]
    output: String,

    /// Color used for heatmap
    #[arg(short, long, value_enum, value_parser, num_args = 0..=3, default_values_t = [Color::Red, Color::Blue, Color::Green])]
    color: Vec<Color>,

    /// Color used for background
    #[arg(short, long, value_enum, value_parser, default_value_t = BackGround::Black)]
    background: BackGround,

    /// Plot accuracy in phred scale
    #[arg(short, long, value_parser, default_value_t = false)]
    phred: bool,

    /// Normalize the counts in each bin with a log2
    #[arg(long, value_parser, default_value_t = false)]
    normalize: bool,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    // check if there are equal number of arguments for the input and color parameters
    if args.input.len() != args.color.len() {
        panic!(
            "\n\nERROR: number of input files ({}) and colors ({}) do not match!",
            args.input.len(),
            args.color.len()
        );
    }
    let transform_accuracy = if args.phred {
        transform::transform_accuracy_phred
    } else {
        transform::transform_accuracy_percent
    };
    let mut hashmaps = vec![];
    for f in args.input {
        utils::is_file(&f).unwrap_or_else(|_| panic!("Input file {f} is invalid",));
        let hashmap = extract_data::bam_to_hashmap(&f, args.threads, transform_accuracy);
        if args.normalize {
            hashmaps.push(extract_data::log_transform_hashmap(hashmap));
        } else {
            hashmaps.push(hashmap);
        }
    }
    plot_heatmap(
        hashmaps,
        args.background,
        args.color,
        &args.output,
        transform_accuracy,
        args.phred,
    );
}

fn max_of_hashmaps(hashmaps: &Vec<HashMap<(usize, usize), i32>>) -> f32 {
    let mut maxes = vec![];
    for h in hashmaps {
        let max_value = *h
            .values()
            .max()
            .expect("ERROR could not get max value of histogram");
        maxes.push(max_value);
    }
    *maxes
        .iter()
        .max()
        .expect("Error getting maximum of hashmaps.") as f32
}

fn reads_to_intensity(
    hashmap: &HashMap<(usize, usize), i32>,
    color: Color,
    maxval: f32,
) -> HashMap<(usize, usize), Array1<u8>> {
    let color = match color {
        Color::Red => arr1(&[1, 0, 0]),
        Color::Green => arr1(&[0, 1, 0]),
        Color::Blue => arr1(&[0, 0, 1]),
        Color::Purple => arr1(&[1, 0, 1]),
        Color::Yellow => arr1(&[1, 1, 0]),
    };
    let mut new_hashmap = HashMap::new();
    for ((length, accuracy), count) in hashmap {
        let intensity = (*count as f32 / maxval * 255.0) as u8;
        let entry = new_hashmap
            .entry((*length, *accuracy))
            .or_insert(arr1(&[0, 0, 0]));
        *entry = color.clone() * intensity;
    }
    new_hashmap
}

fn combine_hashmaps(
    hashmaps: &Vec<HashMap<(usize, usize), i32>>,
    colors: Vec<Color>,
) -> Vec<HashMap<(usize, usize), Array1<u8>>> {
    let maxval = max_of_hashmaps(hashmaps);
    let mut new_hashmaps = vec![];
    for (hashmap, color) in hashmaps.iter().zip(colors) {
        new_hashmaps.push(reads_to_intensity(hashmap, color, maxval));
    }
    new_hashmaps
}

fn plot_heatmap(
    hashmaps: Vec<HashMap<(usize, usize), i32>>,
    background: BackGround,
    color: Vec<Color>,
    output: &str,
    transform_accuracy: fn(f32) -> usize,
    phred: bool,
) {
    let mut image = match background {
        BackGround::Black => RgbImage::from_pixel(601, 601, Rgb([0, 0, 0])),
        BackGround::White => RgbImage::from_pixel(601, 601, Rgb([255, 255, 255])),
    };

    if hashmaps.len() == 1 {
        // Creating a plot with just a single dataset
        info!(
            "Constructing figure with {} colored pixels",
            hashmaps[0].values().len()
        );
        // All counts are scaled to the max value
        let max_value = hashmaps[0]
            .values()
            .max()
            .expect("ERROR could not get max value of histogram");
        // Iterate over the hashmap to fill in bins and color pixels accordingly
        for ((length, accuracy), count) in &hashmaps[0] {
            let intensity = (*count as f32 / *max_value as f32 * 255.0) as u8;
            let color = match color[0] {
                Color::Red => Rgb([intensity, 0, 0]),
                Color::Green => Rgb([0, intensity, 0]),
                Color::Blue => Rgb([0, 0, intensity]),
                Color::Purple => Rgb([intensity, 0, intensity]),
                Color::Yellow => Rgb([intensity, intensity, 0]),
            };
            image.put_pixel(*length as u32, *accuracy as u32, color);
        }
    } else {
        // Creating a plot of multiple datasets
        let default = arr1(&[0, 0, 0]);
        let hashmaps = combine_hashmaps(&hashmaps, color);
        // Iterate over the first hashmap, and call .get for the remaining hashmaps
        // If that bin is unused in one of the remaining hashmaps the default (0, 0, 0) is added
        for ((length, accuracy), arr) in &hashmaps[0] {
            let summed_arr = if hashmaps.len() == 2 {
                arr + hashmaps[1].get(&(*length, *accuracy)).unwrap_or(&default)
            } else {
                arr + hashmaps[1].get(&(*length, *accuracy)).unwrap_or(&default)
                    + hashmaps[2].get(&(*length, *accuracy)).unwrap_or(&default)
            };
            let arr: [u8; 3] = summed_arr.clone().into_raw_vec().try_into().unwrap();
            // Use the summed RGB arrays to fill in the pixel
            image.put_pixel(*length as u32, *accuracy as u32, Rgb(arr));
        }
    }
    info!("Adding axis ticks");
    // Use white text on a black background and vice versa
    image = match background {
        BackGround::Black => {
            axis_ticks::add_ticks(image, transform_accuracy, phred, Rgb([255, 255, 255]))
        }
        BackGround::White => {
            axis_ticks::add_ticks(image, transform_accuracy, phred, Rgb([0, 0, 0]))
        }
    };

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

#[test]
fn test_single_file() {
    let hashmap = extract_data::bam_to_hashmap(
        "test-data/small-test-phased.bam",
        4,
        transform::transform_accuracy_percent,
    );
    plot_heatmap(
        vec![hashmap],
        BackGround::Black,
        vec![Color::Purple],
        "accuracy_heatmap.png",
        transform::transform_accuracy_percent,
        false,
    );
}

#[test]
fn test_single_file_phred() {
    let hashmap = extract_data::bam_to_hashmap(
        "test-data/small-test-phased.bam",
        4,
        transform::transform_accuracy_phred,
    );
    plot_heatmap(
        vec![hashmap],
        BackGround::White,
        vec![Color::Red],
        "accuracy_heatmap.png",
        transform::transform_accuracy_phred,
        true,
    );
}
