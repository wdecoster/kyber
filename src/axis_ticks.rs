use image::{ImageBuffer, Rgb};
use imageproc::{
    drawing::{draw_filled_rect, draw_text},
    rect::Rect,
};
use rusttype::{Font, Scale};

use crate::transform::transform_length;

const SCALE: Scale = Scale { x: 24.0, y: 24.0 };

pub fn add_ticks(
    mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    transform_accuracy: fn(f32) -> usize,
    phred: bool,
    color: Rgb<u8>,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let font_data: &[u8] = include_bytes!("../dev/TimesNewRoman/times new roman.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data).expect("Error parsing font file");

    // add major x-axis ticks at the top and bottom, and add axis labels at the bottom
    for (index, tick) in vec![10, 100, 1000, 10000, 100000].iter().enumerate() {
        let xcoord = transform_length(*tick) as i32;
        let pow = 10i32.pow((index + 1).try_into().unwrap());
        let offset = format!("{tick}").len() as i32;
        image = draw_filled_rect(&image, Rect::at(xcoord, 0).of_size(1, 12), color);
        image = draw_filled_rect(&image, Rect::at(xcoord, 588).of_size(1, 12), color);
        image = draw_text(
            &image,
            color,
            xcoord - 1 - (offset * 5),
            560,
            SCALE,
            &font,
            &format!("{pow}"),
        )
    }

    // adding intermediate x-axis ticks at the top
    for tick in &[5, 50, 500, 5000, 50000, 500000] {
        image = draw_filled_rect(
            &image,
            Rect::at(transform_length(*tick) as i32, 0).of_size(1, 6),
            color,
        );
    }

    // adding minor x-axis ticks at the top
    for tick in 1..10 {
        for m in &[1, 10, 100, 1000, 10000, 100000] {
            image = draw_filled_rect(
                &image,
                Rect::at(transform_length(tick * m) as i32, 0).of_size(1, 2),
                color,
            );
        }
    }

    if phred {
        // add major y-axis ticks left and right, and axis labels on the left
        for tick in &[10, 20, 30] {
            let ycoord = 600 - (15.0 * (*tick as f32)) as i32;
            image = draw_filled_rect(&image, Rect::at(588, ycoord).of_size(12, 1), color);
            image = draw_filled_rect(&image, Rect::at(0, ycoord).of_size(12, 1), color);
            image = draw_text(
                &image,
                color,
                15,
                ycoord - 10,
                SCALE,
                &font,
                &format!("Q{tick}"),
            );
        }

        // add intermediate y-axis ticks
        for tick in &[5.0, 15.0, 25.0, 35.0] {
            image = draw_filled_rect(
                &image,
                Rect::at(594, (15.0 * (*tick as f32)) as i32).of_size(6, 1),
                color,
            );
        }

        // add minor y-axis ticks
        for tick in 0..40 {
            image = draw_filled_rect(
                &image,
                Rect::at(598, (15.0 * (tick as f32)) as i32).of_size(2, 1),
                color,
            );
        }
    } else {
        // add major y-axis ticks  ticks left and right, and axis labels on the left
        for tick in &[80.0, 90.0] {
            let ycoord = transform_accuracy(*tick) as i32;
            image = draw_filled_rect(&image, Rect::at(588, ycoord).of_size(12, 1), color);
            image = draw_filled_rect(&image, Rect::at(0, ycoord).of_size(12, 1), color);
            image = draw_text(
                &image,
                color,
                15,
                ycoord - 10,
                SCALE,
                &font,
                &format!("{tick}%"),
            );
        }
        // add intermediate y-axis ticks
        for tick in &[75.0, 85.0, 95.0] {
            image = draw_filled_rect(
                &image,
                Rect::at(594, transform_accuracy(*tick) as i32).of_size(6, 1),
                color,
            );
        }
        // add minor y-axis ticks
        for tick in 70..100 {
            image = draw_filled_rect(
                &image,
                Rect::at(598, transform_accuracy(tick as f32) as i32).of_size(2, 1),
                color,
            );
        }
    }
    image
}
