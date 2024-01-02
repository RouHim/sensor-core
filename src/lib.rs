use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::{ImageBuffer, ImageFormat, Rgba};
use log::{debug, error};
use serde::{Deserialize, Serialize};

pub mod conditional_image_renderer;
pub mod graph_renderer;
pub mod text_renderer;

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
    /// De/Serialize to PrepareTextData
    PrepareText,
    /// De/Serialize to PrepareStaticImageData
    PrepareStaticImage,
    /// De/Serialize to PrepareConditionalImageData
    PrepareConditionalImage,
    /// De/Serialize to RenderData
    RenderImage,
}

/// Represents the data to be rendered on a display.
/// It holds the display config and the sensor values.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct RenderData {
    pub display_config: DisplayConfig,
    pub sensor_values: Vec<SensorValue>,
}

/// Represents the preparation data for the render process.
/// It holds all static assets to be rendered.
/// This is done once before the loop starts.
/// Each asset will be stored on the display locally, and load during the render process by its
/// asset id / element id
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PrepareTextData {
    /// Key is the element id
    /// Value is the font data
    pub font_data: HashMap<String, Vec<u8>>,
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

/// Represents the display config.
/// It holds the resolution and the elements to be rendered.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct DisplayConfig {
    #[serde(default)]
    pub resolution_height: u32,
    #[serde(default)]
    pub resolution_width: u32,
    #[serde(default)]
    pub elements: Vec<ElementConfig>,
}

/// Represents a single element to be rendered on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct ElementConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub element_type: ElementType,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub text_config: Option<TextConfig>,
    #[serde(default)]
    pub image_config: Option<ImageConfig>,
    #[serde(default)]
    pub graph_config: Option<GraphConfig>,
    #[serde(default)]
    pub conditional_image_config: Option<ConditionalImageConfig>,
}

/// Represents a text element on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct TextConfig {
    #[serde(default)]
    pub sensor_id: String,
    #[serde(default)]
    pub value_modifier: SensorValueModifier,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub font_family: String,
    #[serde(default)]
    pub font_size: u32,
    #[serde(default)]
    pub font_color: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub alignment: TextAlign,
}

/// Represents the text alignment of a text element.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub enum TextAlign {
    #[default]
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "right")]
    Right,
}

/// Represents a static image element on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct ImageConfig {
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub image_path: String,
}

/// Represents the type of a graph element on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub enum GraphType {
    #[default]
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "line-fill")]
    LineFill,
}

/// Represents a graph element on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct GraphConfig {
    #[serde(default)]
    pub sensor_id: String,
    #[serde(default)]
    pub sensor_values: Vec<f64>,
    #[serde(default)]
    pub min_sensor_value: Option<f64>,
    #[serde(default)]
    pub max_sensor_value: Option<f64>,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub graph_type: GraphType,
    #[serde(default)]
    pub graph_color: String,
    #[serde(default)]
    pub graph_stroke_width: i32,
    #[serde(default)]
    pub background_color: String,
    #[serde(default)]
    pub border_color: String,
}

/// Represents a conditional image element on a display.
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct ConditionalImageConfig {
    #[serde(default)]
    pub sensor_id: String,
    #[serde(default)]
    pub sensor_value: String,
    #[serde(default)]
    pub images_path: String,
    #[serde(default)]
    pub min_sensor_value: f64,
    #[serde(default)]
    pub max_sensor_value: f64,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
}

/// Represents the type of an element on a display.
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
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub sensor_type: SensorType,
}

/// Represents the modifier of a sensor value.
/// This is used to modify the value before rendering.
/// For example to output the average or max value of a sensor.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub enum SensorValueModifier {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "avg")]
    Avg,
}

/// Represents the type of a sensor value.
/// This is used to determine how to render the value.
/// For example a text value will be rendered as text, while a number value can be rendered as a graph.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub enum SensorType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "number")]
    Number,
}

/// Render the image
/// The image will be a RGB8 png image
pub fn render_lcd_image(
    display_config: DisplayConfig,
    sensor_value_history: &[Vec<SensorValue>],
    fonts_data: &HashMap<String, Vec<u8>>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let start_time = Instant::now();

    // Get the resolution from the lcd config
    let image_width = display_config.resolution_width;
    let image_height = display_config.resolution_height;

    // Create a new ImageBuffer with the specified resolution
    let mut image = ImageBuffer::new(image_width, image_height);

    // Iterate over lcd elements and draw them on the image
    for lcd_element in display_config.elements {
        draw_element(&mut image, lcd_element, sensor_value_history, fonts_data);
    }

    debug!(" = Total frame render duration: {:?}", start_time.elapsed());

    image
}

/// Draws a single element on the image.
/// The element will be drawn on the given image buffer.
/// Distinguishes between the different element types and calls the corresponding draw function.
fn draw_element(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    lcd_element: ElementConfig,
    sensor_value_history: &[Vec<SensorValue>],
    fonts_data: &HashMap<String, Vec<u8>>,
) {
    let x = lcd_element.x;
    let y = lcd_element.y;
    let element_id = lcd_element.id.as_str();

    // diff between type
    match lcd_element.element_type {
        ElementType::Text => {
            let text_config = lcd_element.text_config.unwrap();
            draw_text(
                image,
                &lcd_element.id,
                text_config,
                x,
                y,
                sensor_value_history,
                fonts_data,
            );
        }
        ElementType::StaticImage => {
            draw_static_image(image, &lcd_element.id, x, y);
        }
        ElementType::Graph => {
            let mut graph_config = lcd_element.graph_config.unwrap();
            graph_config.sensor_values =
                extract_value_sequence(sensor_value_history, &graph_config.sensor_id);

            draw_graph(image, x, y, graph_config);
        }
        ElementType::ConditionalImage => {
            let conditional_image_config = lcd_element.conditional_image_config.unwrap();
            let sensor_value = sensor_value_history[0]
                .iter()
                .find(|&s| s.id == conditional_image_config.sensor_id);
            draw_conditional_image(
                image,
                x,
                y,
                element_id,
                conditional_image_config,
                sensor_value,
            )
        }
    }
}

/// Draws a static image on the image buffer.
fn draw_static_image(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, element_id: &str, x: i32, y: i32) {
    let start_time = Instant::now();

    let cache_dir = get_cache_dir(element_id, &ElementType::StaticImage).join(element_id);
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

    debug!("    - Image render duration: {:?}", start_time.elapsed());
}

fn draw_graph(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: i32, y: i32, config: GraphConfig) {
    let start_time = Instant::now();

    let img_data = graph_renderer::render(&config);
    let graph_image = image::load_from_memory(&img_data).unwrap();

    image::imageops::overlay(image, &graph_image, x as i64, y as i64);

    debug!("    - Graph render duration: {:?}", start_time.elapsed());
}

/// Draws a conditional image on the image buffer.
fn draw_conditional_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    element_id: &str,
    mut config: ConditionalImageConfig,
    sensor_value: Option<&SensorValue>,
) {
    let start_time = Instant::now();

    let sensor_value = match sensor_value {
        None => {
            return;
        }
        Some(sensor_value) => sensor_value,
    };

    config.sensor_value = sensor_value.value.clone();
    let img_data =
        conditional_image_renderer::render(element_id, &sensor_value.sensor_type, &config);

    if let Some(img_data) = img_data {
        let conditional_image = image::load_from_memory(&img_data).unwrap();
        image::imageops::overlay(image, &conditional_image, x as i64, y as i64);
    }

    debug!(
        "    - Conditional image render duration: {:?}",
        start_time.elapsed()
    );
}

/// Draws a text element on the image buffer.
fn draw_text(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    _element_id: &str,
    text_config: TextConfig,
    x: i32,
    y: i32,
    sensor_value_history: &[Vec<SensorValue>],
    fonts_data: &HashMap<String, Vec<u8>>,
) {
    let start_time = Instant::now();

    let font_data = match fonts_data.get(&text_config.font_family) {
        Some(font_data) => font_data,
        None => {
            error!(
                "Font data for font family {} not found",
                text_config.font_family
            );
            return;
        }
    };
    let font = rusttype::Font::try_from_bytes(font_data).unwrap();

    let text_image = text_renderer::render(
        image.width(),
        image.height(),
        &text_config,
        sensor_value_history,
        &font,
    );
    image::imageops::overlay(image, &text_image, x as i64, y as i64);

    debug!("    - Text render duration: {:?}", start_time.elapsed());
}

/// Converts a hex string to a Rgba<u8>
/// The hex string must be in the format #RRGGBBAA
/// Example: #FF0000CC
/// Returns a Rgba<u8> struct
pub fn hex_to_rgba(hex_string: &str) -> Rgba<u8> {
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
pub fn get_cache_dir(element_id: &str, element_type: &ElementType) -> PathBuf {
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
    dirs::cache_dir()
        .unwrap()
        .join(std::env::var("SENSOR_BRIDGE_APP_NAME").unwrap())
}

/// Get the application config dir
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join(std::env::var("SENSOR_BRIDGE_APP_NAME").unwrap())
}
