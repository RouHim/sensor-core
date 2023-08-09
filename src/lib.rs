use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, ImageFormat, Rgba};
use imageproc::drawing;
use lazy_static::lazy_static;
use log::error;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};

pub mod conditional_image_renderer;
pub mod graph_renderer;

/// Indicates the current type of message to be sent to the display.
/// Either a message to prepares static assets, by sending them to the display, and then be stored on the fs.
/// Or the actual render loop, where the prev. stored asses will be used to render the image.
/// The type is used to deserialize the data to the correct struct.
/// The data is a vector of bytes, which will be deserialized to the correct struct, depending on the type.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TransportMessage {
    pub transport_type: TransportType,
    pub data: Vec<u8>,
}

/// Represents the type of the message to be sent to the display.
/// Either a message to prepares static assets, by sending them to the display, and then be stored on the fs.
/// Or the actual render loop, where the prev. stored asses will be used to render the image.
/// The type is used to deserialize the data to the correct struct.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum TransportType {
    /// De/Serialize to PrepareStaticImageData
    PrepareStaticImage,
    /// De/Serialize to PrepareConditionalImageData
    PrepareConditionalImage,
    /// De/Serialize to RenderData
    RenderImage,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct RenderData {
    pub lcd_config: LcdConfig,
    pub sensor_values: Vec<SensorValue>,
}

/// Represents the preparation data for the render process.
/// It holds all static assets to be rendered.
/// This is done once before the loop starts.
/// Each asset will be stored on the display locally, and load during the render process by its
/// asset id / element id
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PrepareStaticImageData {
    /// Key is the element id
    /// Value is the element data
    pub images_data: HashMap<String, Vec<u8>>,
}

/// Represents the preparation data for the render process.
/// It holds all static assets to be rendered.
/// This is done once before the loop starts.
/// Each asset will be stored on the display locally, and load during the render process by its
/// asset id / element id
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PrepareConditionalImageData {
    /// Key is the element id
    /// Value is the element data
    pub images_data: HashMap<String, HashMap<String, Vec<u8>>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct LcdConfig {
    pub resolution_height: u32,
    pub resolution_width: u32,
    pub elements: Vec<LcdElement>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct LcdElement {
    pub id: String,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub element_type: ElementType,
    pub sensor_id: String,
    pub text_config: TextConfig,
    pub image_config: ImageConfig,
    pub graph_config: GraphConfig,
    pub conditional_image_config: ConditionalImageConfig,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct TextConfig {
    pub text_format: String,
    pub font_size: u32,
    pub font_color: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub image_path: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub enum GraphType {
    #[default]
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "line-fill")]
    LineFill,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct GraphConfig {
    pub sensor_values: Vec<f64>,
    pub width: u32,
    pub height: u32,
    pub graph_type: GraphType,
    pub graph_color: String,
    pub graph_stroke_width: i32,
    pub background_color: String,
    pub border_color: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct ConditionalImageConfig {
    pub sensor_id: String,
    pub sensor_value: String,
    pub images_path: String,
    pub min_sensor_value: f64,
    pub max_sensor_value: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
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

/// Provides a single SensorValue
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct SensorValue {
    pub id: String,
    pub value: String,
    pub unit: String,
    pub label: String,
    pub sensor_type: SensorType,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub enum SensorType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "number")]
    Number,
}

const FONT_DATA: &[u8] = include_bytes!("../fonts/FiraCode-Regular.ttf");

/// Render the image
/// The image will be a RGB8 png image
pub fn render_lcd_image(
    lcd_config: LcdConfig,
    sensor_value_history: &[Vec<SensorValue>],
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Get the resolution from the lcd config
    let image_width = lcd_config.resolution_width;
    let image_height = lcd_config.resolution_height;

    // Background color: black, full opacity
    let background_color = Rgba([0, 0, 0, 255]);

    // Create a new ImageBuffer with the specified resolution
    let mut image = ImageBuffer::new(image_width, image_height);
    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = background_color;
    }

    // Draw a simple text on the image using imageproc
    lazy_static! {
        static ref FONT: Font<'static> = Font::try_from_bytes(FONT_DATA).unwrap();
    }

    // Iterate over lcd elements and draw them on the image
    for lcd_element in lcd_config.elements {
        let x = lcd_element.x as i32;
        let y = lcd_element.y as i32;

        // Get the sensor value from the sensor_values Vec by sensor_id
        let element_id = lcd_element.id.as_str();
        let sensor_id = lcd_element.sensor_id.as_str();
        let sensor_value = sensor_value_history[0].iter().find(|&s| s.id == sensor_id);

        // diff between type
        match lcd_element.element_type {
            ElementType::Text => {
                draw_text(
                    &mut image,
                    &FONT,
                    lcd_element.text_config,
                    x,
                    y,
                    sensor_value,
                );
            }
            ElementType::StaticImage => {
                draw_image(&mut image, lcd_element, x, y);
            }
            ElementType::Graph => {
                let mut graph_config = lcd_element.graph_config;
                graph_config.sensor_values =
                    extract_value_sequence(sensor_value_history, lcd_element.sensor_id.as_str());

                draw_graph(&mut image, x, y, graph_config);
            }
            ElementType::ConditionalImage => {
                draw_conditional_image(
                    &mut image,
                    x,
                    y,
                    element_id,
                    lcd_element.conditional_image_config,
                    sensor_value,
                );
            }
        }
    }

    image
}

fn draw_image(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, element: LcdElement, x: i32, y: i32) {
    let cache_dir = get_cache_dir(&element.id, ElementType::StaticImage).join(element.id);
    let file_path = cache_dir.to_str().unwrap();

    if !Path::new(&file_path).exists() {
        error!("File {} does not exist", file_path);
        return;
    }

    // Read image into memory
    // We heavily assume that this is already png encoded to skip the expensive png decoding
    let img_data = fs::read(file_path).unwrap();
    let overlay_image = image::load_from_memory(&img_data).unwrap();

    image::imageops::overlay(image, &overlay_image, x as i64, y as i64);
}

fn draw_graph(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: i32, y: i32, config: GraphConfig) {
    let img_data = graph_renderer::render(config);
    let overlay_image = image::load_from_memory(&img_data).unwrap();

    image::imageops::overlay(image, &overlay_image, x as i64, y as i64);
}

fn draw_conditional_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    element_id: &str,
    mut config: ConditionalImageConfig,
    sensor_value: Option<&SensorValue>,
) {
    let sensor_value = match sensor_value {
        None => {
            return;
        }
        Some(sensor_value) => sensor_value,
    };

    config.sensor_value = sensor_value.value.clone();
    let img_data =
        conditional_image_renderer::render(element_id, &sensor_value.sensor_type, config);

    if let Some(img_data) = img_data {
        let overlay_image = image::load_from_memory(&img_data).unwrap();
        image::imageops::overlay(image, &overlay_image, x as i64, y as i64);
    }
}

fn draw_text(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &Font,
    text_config: TextConfig,
    x: i32,
    y: i32,
    sensor_value: Option<&SensorValue>,
) {
    let font_scale = Scale::uniform(text_config.font_size as f32);
    let font_color: Rgba<u8> = hex_to_color(&text_config.font_color);
    let text_format = text_config.text_format;

    let (value, unit): (&str, &str) = match sensor_value {
        Some(sensor_value) => (&sensor_value.value, &sensor_value.unit),
        _ => ("N/A", ""),
    };

    let text = text_format
        .replace("{value}", value)
        .replace("{unit}", unit);

    drawing::draw_text_mut(image, font_color, x, y, font_scale, font, text.as_str());
}

/// Converts a hex string to a Rgba<u8>
/// The hex string must be in the format #RRGGBBAA
/// Example: #FF0000CC
/// Returns a Rgba<u8> struct
fn hex_to_color(hex_string: &str) -> Rgba<u8> {
    let hex_string = hex_string.trim_start_matches('#');
    let hex = u32::from_str_radix(hex_string, 16).unwrap();
    let r = ((hex >> 24) & 0xff) as u8;
    let g = ((hex >> 16) & 0xff) as u8;
    let b = ((hex >> 8) & 0xff) as u8;
    let a = (hex & 0xff) as u8;
    Rgba([r, g, b, a])
}

/// Extracts the historical values from the sensor_value_history and reverses the order
pub fn extract_value_sequence(
    sensor_value_history: &[Vec<SensorValue>],
    sensor_id: &str,
) -> Vec<f64> {
    let mut sensor_values: Vec<f64> = sensor_value_history
        .iter()
        .flat_map(|history_entry| {
            history_entry.iter().find_map(|entry| {
                if entry.id.eq(sensor_id) {
                    return entry.value.parse().ok();
                }
                None
            })
        })
        .collect();
    sensor_values.reverse();
    sensor_values
}

/// Checks if the given DirEntry is an image
pub fn is_image(dir_entry: &DirEntry) -> bool {
    let entry_path = dir_entry.path();
    let extension_string = entry_path.extension().map(|ext| ext.to_str().unwrap());
    let image_format = extension_string.and_then(ImageFormat::from_extension);
    image_format.map(|x| x.can_read()).unwrap_or(false)
}

/// Get the cache directory for the given element
pub fn get_cache_dir(element_id: &str, element_type: ElementType) -> PathBuf {
    let element_type_folder_name = match element_type {
        ElementType::Text => "text",
        ElementType::StaticImage => "static-image",
        ElementType::Graph => "graph",
        ElementType::ConditionalImage => "conditional-image",
    };

    get_cache_base_dir()
        .join(element_type_folder_name)
        .join(element_id)
}

/// Get the base cache directory
pub fn get_cache_base_dir() -> PathBuf {
    dirs::cache_dir().unwrap().join("sensor-bridge")
}

/// Get the application config dir
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("sensor-bridge")
}
