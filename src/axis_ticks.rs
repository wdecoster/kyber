use image::{ImageBuffer, Rgb};
use imageproc::{drawing::draw_filled_rect, rect::Rect};

use crate::transform::{transform_length, MAX_LENGTH};

pub fn add_ticks(
    mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    transform_accuracy: fn(f32) -> usize,
    phred: bool,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // add major x-axis ticks
    for tick in &[10, 100, 1000, 10000, 100000] {
        image = draw_filled_rect(
            &image,
            Rect::at(transform_length(*tick) as i32, 0).of_size(1, 6),
            Rgb([255, 255, 255]),
        );
    }
    // adding intermediate x-axis ticks
    for tick in &[5, 50, 500, 5000, 50000, 500000] {
        image = draw_filled_rect(
            &image,
            Rect::at(transform_length(*tick) as i32, 0).of_size(1, 3),
            Rgb([255, 255, 255]),
        );
    }
    // adding minor x-axis ticks
    for tick in 1..10 {
        for multiplier in &[1, 10, 100, 1000, 10000, 100000] {
            image = draw_filled_rect(
                &image,
                Rect::at(transform_length(tick * multiplier) as i32, 0).of_size(1, 1),
                Rgb([255, 255, 255]),
            );
        }
    }
    if phred {
        // add major y-axis ticks
        for tick in &[10, 20, 30, 40] {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 6,
                    (7.5 * (*tick as f32)) as i32,
                )
                .of_size(6, 1),
                Rgb([255, 255, 255]),
            );
        }
        // add intermediate y-axis ticks
        for tick in &[5.0, 15.0, 25.0, 35.0] {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 3,
                    (7.5 * (*tick as f32)) as i32,
                )
                .of_size(3, 1),
                Rgb([255, 255, 255]),
            );
        }
        // add minor y-axis ticks
        for tick in 0..40 {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 1,
                    (7.5 * (tick as f32)) as i32,
                )
                .of_size(1, 1),
                Rgb([255, 255, 255]),
            );
        }
    } else {
        // add major y-axis ticks
        for tick in &[70.0, 80.0, 90.0, 100.0] {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 6,
                    transform_accuracy(*tick) as i32,
                )
                .of_size(6, 1),
                Rgb([255, 255, 255]),
            );
        }
        // add intermediate y-axis ticks
        for tick in &[75.0, 85.0, 95.0] {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 3,
                    transform_accuracy(*tick) as i32,
                )
                .of_size(3, 1),
                Rgb([255, 255, 255]),
            );
        }
        // add minor y-axis ticks
        for tick in 70..100 {
            image = draw_filled_rect(
                &image,
                Rect::at(
                    transform_length(MAX_LENGTH) as i32 - 1,
                    transform_accuracy(tick as f32) as i32,
                )
                .of_size(1, 1),
                Rgb([255, 255, 255]),
            );
        }
    }
    image
}
