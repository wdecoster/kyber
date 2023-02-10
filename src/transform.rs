const RESOLUTION_FACTOR: f32 = 5.0;
const _MIN_LENGTH: usize = 10;
pub(crate) const MAX_LENGTH: usize = 1000000;
pub(crate) const MIN_IDENTITY: f32 = 70.0;

pub fn transform_length(seqlen: usize) -> usize {
    ((seqlen as f32).log10() * 10.0 * RESOLUTION_FACTOR) as usize
}

pub fn transform_accuracy_percent(identity: f32) -> usize {
    (RESOLUTION_FACTOR * 2.0 * (100.0 - identity)) as usize
}

pub fn transform_accuracy_phred(identity: f32) -> usize {
    (RESOLUTION_FACTOR * 2.0 * (100.0 - identity)) as usize
}
