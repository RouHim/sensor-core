use image::{ImageBuffer, Rgba};
use imageproc::drawing;
use rusttype::Font;

use crate::{hex_to_rgba, SensorType, SensorValue, SensorValueModifier, TextAlign, TextConfig};

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
    sensor_value_history: &[Vec<SensorValue>],
    font: &Font,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Initialize image buffer
    let font_scale = rusttype::Scale::uniform(text_config.font_size as f32);
    let font_color: Rgba<u8> = hex_to_rgba(&text_config.font_color);
    let sensor_id = &text_config.sensor_id;

    // Replace placeholders in text format
    let text = replace_placeholders(text_config, sensor_id, sensor_value_history);

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
    let y: u32 = if text_config.height > text_image.height() {
        (text_config.height - text_image.height()) / 2
    } else {
        0
    };
    let x: u32 = if text_config.width > text_image.width() {
        text_config.width - text_image.width()
    } else {
        0
    };
    match text_config.alignment {
        TextAlign::Left => {
            image::imageops::overlay(&mut image, &text_image, 0, y as i64);
        }
        TextAlign::Center => {
            let x = x / 2;
            image::imageops::overlay(&mut image, &text_image, x as i64, y as i64);
        }
        TextAlign::Right => {
            image::imageops::overlay(&mut image, &text_image, x as i64, y as i64);
        }
    }

    image
}

/// Replaces the placeholders in the text format with the actual values
/// FIXME: The special placeholders like {value-avg} may be calculated multiple times
///        This is not a problem for now because 95% of the time they are not or rarely used
///        But if we encounter performance issues, we should optimize this (Esp. if the history is long)
fn replace_placeholders(
    text_config: &TextConfig,
    sensor_id: &str,
    sensor_value_history: &[Vec<SensorValue>],
) -> String {
    let mut text_format = text_config.format.clone();

    if text_format.contains("{value-avg}") {
        text_format = text_format.replace(
            "{value-avg}",
            get_value_avg(sensor_id, sensor_value_history).as_str(),
        );
    }

    if text_format.contains("{value-min}") {
        text_format = text_format.replace(
            "{value-min}",
            get_value_min(sensor_id, sensor_value_history).as_str(),
        );
    }

    if text_format.contains("{value-max}") {
        text_format = text_format.replace(
            "{value-max}",
            get_value_max(sensor_id, sensor_value_history).as_str(),
        );
    }

    if text_format.contains("{value}") {
        let value = match text_config.value_modifier {
            SensorValueModifier::None => get_value(sensor_id, sensor_value_history),
            SensorValueModifier::Avg => get_value_avg(sensor_id, sensor_value_history),
            SensorValueModifier::Max => get_value_max(sensor_id, sensor_value_history),
            SensorValueModifier::Min => get_value_min(sensor_id, sensor_value_history),
        };
        text_format = text_format.replace("{value}", value.as_str());
    }

    if text_format.contains("{unit}") {
        text_format =
            text_format.replace("{unit}", get_unit(sensor_id, sensor_value_history).as_str());
    }

    text_format
}

/// Returns the sensor unit of the latest sensor value
fn get_unit(sensor_id: &str, sensor_value_history: &[Vec<SensorValue>]) -> String {
    match get_latest_value(sensor_id, sensor_value_history) {
        Some(value) => value.unit,
        None => "".to_string(),
    }
}

// Returns the latest sensor value
fn get_value(sensor_id: &str, sensor_value_history: &[Vec<SensorValue>]) -> String {
    match get_latest_value(sensor_id, sensor_value_history) {
        Some(value) => value.value,
        None => "N/A".to_string(),
    }
}

/// Returns the minimum sensor value of all sensor values in the history
fn get_value_min(sensor_id: &str, sensor_value_history: &[Vec<SensorValue>]) -> String {
    let number_values_history = get_sensor_values_as_number(sensor_id, sensor_value_history);

    // If there are no values, return N/A
    if number_values_history.is_empty() {
        return "N/A".to_string();
    }

    // Get the minimum value
    let min = number_values_history
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    format!("{:.2}", min).to_string()
}

/// Returns the maximum sensor value of all sensor values in the history
fn get_value_max(sensor_id: &str, sensor_value_history: &[Vec<SensorValue>]) -> String {
    let number_values_history = get_sensor_values_as_number(sensor_id, sensor_value_history);

    // If there are no values, return N/A
    if number_values_history.is_empty() {
        return "N/A".to_string();
    }

    // Get the maximum value
    let max = number_values_history
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    format!("{:.2}", max).to_string()
}

/// Returns the average sensor value of all sensor values in the history
fn get_value_avg(sensor_id: &str, sensor_value_history: &[Vec<SensorValue>]) -> String {
    let number_values_history = get_sensor_values_as_number(sensor_id, sensor_value_history);

    // If there are no values, return N/A
    if number_values_history.is_empty() {
        return "N/A".to_string();
    }

    let avg = number_values_history.iter().sum::<f64>() / number_values_history.len() as f64;

    format!("{:.2}", avg).to_string()
}

fn get_sensor_values_as_number(
    sensor_id: &str,
    sensor_value_history: &[Vec<SensorValue>],
) -> Vec<f64> {
    let values = sensor_value_history
        .iter()
        .flat_map(|sensor_values| sensor_values.iter().find(|&s| s.id == sensor_id))
        .filter(|sensor_value| sensor_value.sensor_type == SensorType::Number)
        .map(|sensor_value| sensor_value.value.parse::<f64>().unwrap())
        .collect::<Vec<f64>>();
    values
}

fn get_latest_value(
    sensor_id: &str,
    sensor_value_history: &[Vec<SensorValue>],
) -> Option<SensorValue> {
    sensor_value_history[0]
        .iter()
        .find(|&s| s.id == sensor_id)
        .cloned()
}

/// Calculates the bounding box of the text in the image
/// This is done by detecting the first and last non-transparent pixel in each direction
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
