use std::io::{BufWriter, Cursor};

use image::{ImageBuffer, Rgba, RgbaImage};

use crate::{hex_to_rgba, GraphConfig, GraphType};

/// Renders a graph based on the given config
/// # Returns
/// A vector of bytes containing the RGB8 png image
/// # Arguments
/// * `graph_config` - The config for the graph
pub fn render(graph_config: &GraphConfig) -> Vec<u8> {
    let width = graph_config.width;
    let height = graph_config.height;

    // Prepare the data for the graph
    let graph_data = prepare_graph_data(width, &graph_config.sensor_values);

    // Render the graph
    let mut image = match graph_config.graph_type {
        GraphType::Line => render_line_chart(&graph_data, graph_config),
        GraphType::LineFill => render_line_chart_filled(&graph_data, graph_config),
    };

    // Draw border if border is visible
    if !graph_config.border_color.ends_with("00") {
        draw_border(&mut image, &graph_config.border_color, width, height);
    }

    // Encode to png and return encoded bytes
    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
    image
        .write_to(&mut writer, image::ImageOutputFormat::Png)
        .unwrap();

    writer.into_inner().unwrap().into_inner()
}

/// Draws a border around the specified image
fn draw_border(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    border_color: &str,
    width: u32,
    height: u32,
) {
    let border_color = hex_to_rgba(border_color);
    let border_x: i32 = 0;
    let border_y: i32 = 0;
    let border_width: u32 = width;
    let border_height: u32 = height;
    imageproc::drawing::draw_hollow_rect_mut(
        image,
        imageproc::rect::Rect::at(border_x, border_y).of_size(border_width, border_height),
        border_color,
    );
}

/// Prepares the plot data for the graph.
/// Aligns the sensor values to the width of the desired graph width.
fn prepare_graph_data(width: u32, sensor_values: &Vec<f64>) -> Vec<f64> {
    // Ensure that sensor values does not exceed the width, if so cut them and keep the last values
    let sensor_values = if sensor_values.len() > width as usize {
        sensor_values[(sensor_values.len() - width as usize)..].to_vec()
    } else {
        sensor_values.to_vec()
    };

    // Create a new vector for the width of the image, initialize with 0
    let mut plot_data: Vec<f64> = vec![0.0; (width) as usize];

    // Set the gen values to the end of plat data
    plot_data.splice(
        (width - sensor_values.len() as u32) as usize..,
        sensor_values,
    );
    plot_data
}

/// Renders a graph based on the given config
fn render_line_chart(numbers: &[f64], config: &GraphConfig) -> RgbaImage {
    let width = config.width;
    let height = config.height;
    let min_value = config.min_sensor_value.unwrap_or(get_min(numbers));
    let max_value = config.max_sensor_value.unwrap_or(get_max(numbers));
    let line_width = config.graph_stroke_width;
    let line_color = hex_to_rgba(&config.graph_color);
    let background_color = hex_to_rgba(&config.background_color);

    let mut image = RgbaImage::from_pixel(width, height, background_color);
    let half_line_width = (line_width / 2) as f32;

    for i in 0..numbers.len() - 1 {
        let current_value = numbers[i];
        let next_value = numbers[i + 1];

        // First move value between 0 and 1, where min_value is the lower bound and max_value the upper bound
        let current_value_normalized = (current_value - min_value) / (max_value - min_value);
        let next_value_normalized = (next_value - min_value) / (max_value - min_value);

        // Then move the value between 0 and height
        let img_line_start = current_value_normalized * height as f64;
        let img_line_end = next_value_normalized * height as f64;

        // Render line on image
        let x0 = i;
        let y0 = height - img_line_start as u32;

        let x1 = i + 1;
        let y1 = height - img_line_end as u32;

        // Draw graph line
        for offset in -half_line_width as i32..=half_line_width as i32 {
            imageproc::drawing::draw_line_segment_mut(
                &mut image,
                ((x0 as f32) + offset as f32, y0 as f32),
                ((x1 as f32) + offset as f32, y1 as f32),
                line_color,
            );
        }
    }

    image
}

/// Renders a graph based on the given config
fn render_line_chart_filled(numbers: &[f64], config: &GraphConfig) -> RgbaImage {
    let width = config.width;
    let height = config.height;
    let min_value = config.min_sensor_value.unwrap_or(get_min(numbers));
    let max_value = config.max_sensor_value.unwrap_or(get_max(numbers));
    let line_width = config.graph_stroke_width;
    let line_color = hex_to_rgba(&config.graph_color);
    let fill_color = line_color;
    let background_color = hex_to_rgba(&config.background_color);

    let mut image = RgbaImage::from_pixel(width, height, background_color);
    let half_line_width = (line_width / 2) as f32;

    for i in 0..numbers.len() - 1 {
        let current_value = numbers[i];
        let next_value = numbers[i + 1];

        // First move value between 0 and 1, where min_value is the lower bound and max_value the upper bound
        let current_value_normalized = (current_value - min_value) / (max_value - min_value);
        let next_value_normalized = (next_value - min_value) / (max_value - min_value);

        // Then move the value between 0 and height
        let img_line_start = current_value_normalized * height as f64;
        let img_line_end = next_value_normalized * height as f64;

        // Render line on image
        let x0 = i;
        let y0 = height - img_line_start as u32;

        let x1 = i + 1;
        let y1 = height - img_line_end as u32;

        // Fill the area under the line until image bottom, respect the line width
        let mut y = y0;
        while y < height {
            image.put_pixel(x0 as u32, y, fill_color);
            y += 1;
        }

        // Draw graph line
        for offset in -half_line_width as i32..=half_line_width as i32 {
            imageproc::drawing::draw_line_segment_mut(
                &mut image,
                ((x0 as f32) + offset as f32, y0 as f32),
                ((x1 as f32) + offset as f32, y1 as f32),
                line_color,
            );
        }
    }

    image
}

/// Returns the minimum value of the given vector
fn get_min(values: &[f64]) -> f64 {
    let mut min = values[0];
    for value in values {
        if *value < min {
            min = *value;
        }
    }
    min
}

/// Returns the maximum value of the given vector
fn get_max(values: &[f64]) -> f64 {
    let mut max = values[0];
    for value in values {
        if *value > max {
            max = *value;
        }
    }
    max
}
