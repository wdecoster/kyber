use rust_htslib::bam::{
    self,
    record::{Aux, Cigar},
};

/// Calculates the gap-compressed identity
/// based on https://lh3.github.io/2018/11/25/on-the-definition-of-sequence-identity
/// recent minimap2 version have that as the de tag
/// if that is not present it is calculated from CIGAR and NM
pub fn gap_compressed_identity(record: std::rc::Rc<rust_htslib::bam::Record>) -> f32 {
    match get_de_tag(&record) {
        Some(v) => v,
        None => {
            let mut matches = 0;
            let mut gap_size = 0;
            let mut gap_count = 0;
            for entry in record.cigar().iter() {
                match entry {
                    Cigar::Match(len) | Cigar::Equal(len) | Cigar::Diff(len) => {
                        matches += *len;
                    }
                    Cigar::Del(len) | Cigar::Ins(len) => {
                        gap_size += *len;
                        gap_count += 1;
                    }
                    _ => (),
                }
            }
            100.0 * (1.0 - ((get_nm_tag(&record) - gap_size + gap_count) as f32
                / (matches + gap_count) as f32))
        }
    }
}

fn get_nm_tag(record: &bam::Record) -> u32 {
    match record.aux(b"NM") {
        Ok(value) => match value {
            Aux::U8(v) => u32::from(v),
            Aux::U16(v) => u32::from(v),
            Aux::U32(v) => v,
            _ => panic!("Unexpected type of Aux {value:?}"),
        },
        Err(_e) => panic!("Unexpected result while trying to access the NM tag"),
    }
}

/// Get the de:f tag from minimap2, which is the gap compressed sequence divergence
/// Which is converted into percent identity with 100 * (1 - de)
/// This tag can be absent if the aligner version is not quite recent
fn get_de_tag(record: &bam::Record) -> Option<f32> {
    match record.aux(b"de") {
        Ok(value) => match value {
            Aux::Float(v) => Some(100.0 * (1.0 - v)),
            _ => panic!("Unexpected type of Aux {value:?}"),
        },
        Err(_e) => None,
    }
}


pub fn ubam_accuracy(record: std::rc::Rc<rust_htslib::bam::Record>) -> f32 {
    // get the expected accuracy from the quality scores in the bam file
    // for this, convert each quality score to the error probability
    // and calculate the average error probability
    let mut error_probabilities = Vec::new();
    for quality in record.qual().iter() {
        error_probabilities.push(10.0f32.powf(-(*quality as f32) / 10.0));
    }
    100.0 * (1.0 - error_probabilities.iter().sum::<f32>() / error_probabilities.len() as f32)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_record_with_qual(qual: &[u8]) -> std::rc::Rc<bam::Record> {
        let mut record = bam::Record::new();
        // create a seq with the same length as the quality scores
        let seq= vec![b'A'; qual.len()];
        record.set(&[], None, &seq, qual);
        std::rc::Rc::new(record)
    }

    #[test]
    fn test_ubam_accuracy() {
        let record = create_record_with_qual(&[30, 30, 30, 30, 30]);
        let accuracy = ubam_accuracy(record);
        assert!((accuracy - 99.9).abs() < f32::EPSILON);

        let record = create_record_with_qual(&[20, 20, 20, 20, 20]);
        let accuracy = ubam_accuracy(record);
        assert!((accuracy - 99.0).abs() < f32::EPSILON);

        let record = create_record_with_qual(&[10, 10, 10, 10, 10]);
        let accuracy = ubam_accuracy(record);
        assert!((accuracy - 90.0).abs() < f32::EPSILON);
    }
}
