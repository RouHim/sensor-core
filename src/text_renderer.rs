use image::{ImageBuffer, Rgba};
use imageproc::drawing;

use crate::{hex_to_rgba, SensorValue, TextAlign, TextConfig};
use rusttype::Font;

/// Renders the text element to a png image.
/// Render Pipeline:
///     1. Draw text on empty rgba buffer on display size
///     2. Calculate bounding box of text
///     3. Crop buffer to the visible bounding box of the text
///     4. Create a new Image buffer in the size of the text element
///     5. Overlay the text image on the new image buffer according to the text alignment
pub fn render(
    image_width: u32,
    image_height: u32,
    text_config: &TextConfig,
    sensor_value: Option<&SensorValue>,
    font: &Font,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Initialize image buffer
    let font_scale = rusttype::Scale::uniform(text_config.font_size as f32);
    let font_color: Rgba<u8> = hex_to_rgba(&text_config.font_color);
    let text_format = &text_config.format;

    let (value, unit): (&str, &str) = match sensor_value {
        Some(sensor_value) => (&sensor_value.value, &sensor_value.unit),
        _ => ("N/A", ""),
    };

    // Replace placeholders in text format
    let text = text_format
        .replace("{value}", value)
        .replace("{unit}", unit);

    let mut image = image::RgbaImage::new(image_width, image_height);

    // 1. Draw text on empty rgba buffer on display size
    drawing::draw_text_mut(
        &mut image,
        font_color,
        25,
        7,
        font_scale,
        font,
        text.as_str(),
    );

    // 2. Calculate bounding box of text
    let text_bounding_box = get_bounding_box(&image);

    // 3. Crop buffer to the visible bounding box of the text
    let text_image = image::imageops::crop(
        &mut image,
        text_bounding_box.left() as u32,
        text_bounding_box.top() as u32,
        text_bounding_box.width(),
        text_bounding_box.height(),
    )
    .to_image();

    // 4. Create a new Image buffer in the size of the text element
    let mut image = image::RgbaImage::new(text_config.width, text_config.height);

    // 5. Overlay the text image on the new image buffer according to the text alignment
    // Center text vertically
    let y = (text_config.height - text_image.height()) / 2;
    match text_config.alignment {
        TextAlign::Left => {
            image::imageops::overlay(&mut image, &text_image, 0, y as i64);
        }
        TextAlign::Center => {
            let x = (text_config.width - text_image.width()) / 2;
            image::imageops::overlay(&mut image, &text_image, x as i64, y as i64);
        }
        TextAlign::Right => {
            let x = text_config.width - text_image.width();
            image::imageops::overlay(&mut image, &text_image, x as i64, y as i64);
        }
    }

    image
}

/// Calculates the bounding box of the text in the image
fn get_bounding_box(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> imageproc::rect::Rect {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = image.width();
    let mut max_y = image.height();

    // Detect bounding box from left
    for x in 0..image.width() {
        let mut line_empty = true;
        for y in 0..image.height() {
            let pixel = image.get_pixel(x, y);
            if pixel != &Rgba([0, 0, 0, 0]) {
                line_empty = false;
                break;
            }
        }

        if !line_empty {
            min_x = x;
            break;
        }
    }

    // Detect bounding box from top
    for y in 0..image.height() {
        let mut line_empty = true;
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            if pixel != &Rgba([0, 0, 0, 0]) {
                line_empty = false;
                break;
            }
        }

        if !line_empty {
            min_y = y - 1;
            break;
        }
    }

    // Detect bounding box from right
    for x in (0..image.width()).rev() {
        let mut line_empty = true;
        for y in (0..image.height()).rev() {
            let pixel = image.get_pixel(x, y);
            if pixel != &Rgba([0, 0, 0, 0]) {
                line_empty = false;
                break;
            }
        }

        if !line_empty {
            max_x = x + 1;
            break;
        }
    }

    // Detect bounding box from bottom
    for y in (0..image.height()).rev() {
        let mut line_empty = true;
        for x in (0..image.width()).rev() {
            let pixel = image.get_pixel(x, y);
            if pixel != &Rgba([0, 0, 0, 0]) {
                line_empty = false;
                break;
            }
        }

        if !line_empty {
            max_y = y + 1;
            break;
        }
    }

    imageproc::rect::Rect::at(min_x as i32, min_y as i32).of_size(max_x - min_x, max_y - min_y)
}
