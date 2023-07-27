use std::fmt;

use image::DynamicImage::ImageRgb8;
use image::{ImageBuffer, Rgb, RgbImage};
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LcdConfig {
    pub resolution_height: u32,
    pub resolution_width: u32,
    pub elements: Vec<LcdElement>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LcdElement {
    pub id: String,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub element_type: ElementType,
    pub sensor_id: String,
    pub text_format: String,
    pub font_size: u32,
    pub font_color: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum ElementType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "static-image")]
    StaticImage,
    #[serde(rename = "graph")]
    Graph,
    #[serde(rename = "conditional-image")]
    ConditionalImage,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TransferData {
    pub lcd_config: LcdConfig,
    pub sensor_values: Vec<SensorValue>,
}

/// Render the image
/// The image will be a RGB8 png image
///
pub fn render_lcd_image(lcd_config: LcdConfig, sensor_values: Vec<SensorValue>) -> RgbImage {
    // Get the resolution from the lcd config
    let image_width = lcd_config.resolution_width;
    let image_height = lcd_config.resolution_height;

    // Background color; black
    let background_color = Rgb([0, 0, 0]);

    // Create a new ImageBuffer with the specified resolution
    let mut image = ImageBuffer::new(image_width, image_height);
    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = background_color;
    }

    // Draw a simple text on the image using imageproc
    let font_data = Vec::from(include_bytes!("../fonts/FiraCode-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font_data).unwrap();

    // Iterate over lcd elements and draw them on the image
    for lcd_element in lcd_config.elements {
        let x = lcd_element.x as i32;
        let y = lcd_element.y as i32;

        // Get the sensor value from the sensor_values Vec by sensor_id
        let sensor_id = lcd_element.sensor_id.as_str();
        let sensor_value = sensor_values.iter().find(|&s| s.id == sensor_id);

        // diff between type
        match lcd_element.element_type {
            ElementType::Text => {
                draw_text(&mut image, &font, lcd_element, x, y, sensor_value);
            }
            ElementType::StaticImage => {}
            ElementType::Graph => {}
            ElementType::ConditionalImage => {}
        }
    }

    // Convert the ImageBuffer to a DynamicImage RGB8
    let dynamic_img = ImageRgb8(image);

    dynamic_img.to_rgb8()
}

fn draw_text(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    font: &Font,
    lcd_element: LcdElement,
    x: i32,
    y: i32,
    sensor_value: Option<&SensorValue>,
) {
    let font_scale = Scale::uniform(lcd_element.font_size as f32);
    let font_color: Rgb<u8> = hex_to_color(&lcd_element.font_color);
    let text_format = lcd_element.text_format;

    let (value, unit): (&str, &str) = match sensor_value {
        Some(sensor_value) => (&sensor_value.value, &sensor_value.unit),
        _ => ("N/A", ""),
    };

    let text = text_format
        .replace("{value}", value)
        .replace("{unit}", unit);

    imageproc::drawing::draw_text_mut(image, font_color, x, y, font_scale, font, text.as_str());
}

/// Converts a hex string to a Rgb<u8>
/// The hex string must be in the format #RRGGBB
/// Example: #FF0000
/// Returns a Rgb<u8> struct
fn hex_to_color(string: &str) -> Rgb<u8> {
    let hex = string.trim_start_matches('#');
    let rgb = u32::from_str_radix(hex, 16).unwrap();
    Rgb([
        (rgb >> 16) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    ])
}

/// Provides a single SensorValue
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SensorValue {
    pub id: String,
    pub value: String,
    pub unit: String,
    pub label: String,
    pub sensor_type: String,
}

/// Renders a SensorValue to a string
impl fmt::Display for SensorValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id: {}, value: {}, label: {} type: {}",
            self.id, self.value, self.label, self.sensor_type
        )
    }
}
