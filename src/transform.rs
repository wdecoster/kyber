// The transformations below and the minimal and maximal cutoffs
// below make sure that both lengths and accuracies end up in an equal space
// the current parameters result in a 600*600 image

use std::cmp::{max, min};

const RESOLUTION_FACTOR: f32 = 10.0;
const _MIN_LENGTH: usize = 10;
pub(crate) const MAX_LENGTH: usize = 1000000;
pub(crate) const MIN_IDENTITY: f32 = 70.0;
const MAX_PHRED: f32 = 40.0;

// log10-transform the read lengths, which are limited to 1M reads
// log10(1M) = 6, multiply by 50 to get a 600 pixels axis
pub fn transform_length(seqlen: usize) -> usize {
    min(
        ((MAX_LENGTH as f32).log10() * 10.0 * RESOLUTION_FACTOR) as usize,
        ((seqlen as f32).log10() * 10.0 * RESOLUTION_FACTOR) as usize,
    )
}

// identities are converted to error rate to start the plot from the top left corner
// minimal accuracy is 70, so the 30 accuracies levels (after cast to usize)
// are multiplied by 20 to get a 600 pixels axis
pub fn transform_accuracy_percent(identity: f32) -> usize {
    min(
        (RESOLUTION_FACTOR * 2.0 * (100.0 - MIN_IDENTITY)) as usize,
        (RESOLUTION_FACTOR * 2.0 * (100.0 - identity)) as usize,
    )
}

// identities are converted to phred scale and capped at 40
// this leaves us with 40 accuracy levels (after cast to usize)
// and this is multiplied by 7.5 to get a 600 pixels axis
pub fn transform_accuracy_phred(identity: f32) -> usize {
    max(
        0,
        (15.0 * (MAX_PHRED - accuracy_to_phred(identity))) as usize,
    )
}

fn accuracy_to_phred(identity: f32) -> f32 {
    -10.0 * (1.0 - identity / 100.0).log10()
}

#[test]
fn test_accuracy_to_phred() {
    assert!((accuracy_to_phred(90.0) - 10.0).abs() < 0.01);
    assert!((accuracy_to_phred(99.0) - 20.0).abs() < 0.01);
    assert!((accuracy_to_phred(99.9) - 30.0).abs() < 0.01);
}
