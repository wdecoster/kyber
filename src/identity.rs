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
            1.0 - ((get_nm_tag(&record) - gap_size + gap_count) as f32
                / (matches + gap_count) as f32)
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
