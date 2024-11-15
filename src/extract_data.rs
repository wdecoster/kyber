use std::collections::HashMap;

use log::info;
use rust_htslib::{
    bam::{self, Read},
    htslib,
};

use crate::{identity, transform};

pub fn bam_to_hashmap(
    bam_file: &str,
    threads: usize,
    transform_accuracy: fn(f32) -> usize,
) -> HashMap<(usize, usize), i32> {
    let mut bam = if bam_file == "-" {
        bam::Reader::from_stdin().expect("\n\nError reading alignments from stdin.\nDid you include the file header with samtools view -h?\n\n\n\n")
    } else {
        bam::Reader::from_path(bam_file)
            .expect("Error opening BAM/CRAM file.\nIs the input file correct?\n\n\n\n")
    };
    bam.set_threads(threads)
        .expect("Failure setting decompression threads");
    let histogram = bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
        .fold(HashMap::new(), |mut hist, record| {
            let length = transform::transform_length(record.seq_len());
            let error = transform_accuracy(identity::gap_compressed_identity(record));
            let entry = hist.entry((length, error)).or_insert(0);
            *entry += 1;
            hist
        });

    info!("Constructed hashmap for histogram");
    if histogram.is_empty() {
        panic!("No reads found in BAM file {}", bam_file);
    }
    histogram
}

pub fn log_transform_hashmap(
    hashmap: HashMap<(usize, usize), i32>,
) -> HashMap<(usize, usize), i32> {
    let mut transformed_hashmap = HashMap::new();
    for ((key, value), count) in hashmap {
        transformed_hashmap.insert((key, value), (count as f32).log2() as i32);
    }
    transformed_hashmap
}
