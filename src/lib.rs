use serde::{Deserialize, Serialize};
use image::{ImageBuffer, ImageOutputFormat};
use rusttype::{Font, Scale};
use image::DynamicImage::ImageRgb8;
use std::io::{Cursor, Seek, SeekFrom};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LcdConfig {
    pub resolution_height: u32,
    pub resolution_width: u32,
    pub elements: Vec<LcdElement>,
}

impl LcdConfig {
    fn default() -> LcdConfig {
        LcdConfig {
            resolution_height: 512,
            resolution_width: 512,
            elements: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LcdElement {
    pub id: String,
    pub name: String,
    pub sensor_id: String,
    pub text_format: String,
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SerialTransferData {
    pub lcd_config: LcdConfig,
    pub sensor_values: Vec<SensorValue>,
}

/// Render the image
/// The image will be a RGB8 png image
///
pub fn render_lcd_image(lcd_config: LcdConfig, sensor_values: Vec<SensorValue>) -> Vec<u8> {
// Get the resolution from the lcd config
    let image_width = lcd_config.resolution_width;
    let image_height = lcd_config.resolution_height;

    // Create a new ImageBuffer with the specified resolution only in black
    let mut image = ImageBuffer::new(image_width, image_height);
    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = image::Rgb([0, 0, 0]);
    }

    // Draw a simple text on the image using imageproc
    let font_data = Vec::from(include_bytes!("../fonts/Roboto-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font_data).unwrap();
    let font_scale = Scale::uniform(20.0);
    let font_color = image::Rgb([255, 255, 255]);


    // Iterate over lcd elements and draw them on the image
    for lcd_element in lcd_config.elements {
        let x = lcd_element.x as i32;
        let y = lcd_element.y as i32;
        let sensor_id = lcd_element.sensor_id.as_str();
        let text_format = lcd_element.text_format;

        // Get the sensor value from the sensor_values Vec by sensor_id
        let sensor_value = sensor_values.iter().find(|&s| s.id == sensor_id).unwrap();

        let value = sensor_value.value.as_str();
        let unit = sensor_value.unit.as_str();
        let text = text_format
            .replace("{value}", value)
            .replace("{unit}", unit);

        imageproc::drawing::draw_text_mut(
            &mut image,
            font_color,
            x,
            y,
            font_scale,
            &font,
            text.as_str(),
        );
    }

    // Convert the ImageBuffer to a DynamicImage RGB8
    let dynamic_img = ImageRgb8(image);

    // Create a Vec<u8> buffer to write the image to it
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    dynamic_img
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .unwrap();

    // Reset the cursor to the beginning of the buffer
    cursor.seek(SeekFrom::Start(0)).unwrap();
    buf
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
