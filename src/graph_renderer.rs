use std::io::{BufWriter, Cursor};

use image::{ImageBuffer, Rgba};
use imageproc::drawing;
use poloto::build;
use poloto::build::markers;
use resvg::tiny_skia::{Color, Pixmap};
use resvg::usvg::{Align, NodeExt, Options, TreeParsing};
use resvg::{tiny_skia, usvg};
use tagu::prelude::Elem;

use crate::{hex_to_rgba, GraphConfig, GraphType};

/// Renders a graph based on the given config
/// # Returns
/// A vector of bytes containing the RGB8 png image
/// # Arguments
/// * `graph_config` - The config for the graph
pub fn render(graph_config: &GraphConfig) -> Vec<u8> {
    let width = graph_config.width;
    let height = graph_config.height;

    // Create graph
    let graph_data = prepare_graph_data(width, &graph_config.sensor_values);
    let svg_data = render_to_svg(graph_data, graph_config);
    let graph_pixmap = render_to_raster(&svg_data, width, height, &graph_config.background_color);

    // Copy pixmap to image buffer
    let mut image = image::ImageBuffer::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let pixel_color = graph_pixmap.pixel(x, y).unwrap();
        *pixel = Rgba([
            pixel_color.red(),
            pixel_color.green(),
            pixel_color.blue(),
            pixel_color.alpha(),
        ]);
    }

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
    drawing::draw_hollow_rect_mut(
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

/// Renders the given svg data to a png image
/// # Returns
/// SVG data as String
fn render_to_svg(some_numbers: Vec<f64>, config: &GraphConfig) -> String {
    // Because we are going to extract only the path from the svg,
    // we should render the total plot greater than the actual desired image
    // So we get a decent quality
    let plot_width = config.width as f64 * 2.0;
    let plot_height = config.height as f64 * 2.0;

    let line_data: Vec<[f64; 2]> = some_numbers
        .iter()
        .enumerate()
        .map(|(i, x)| [i as f64, *x])
        .collect();

    let line_plot = match config.graph_type {
        GraphType::Line => build::plot("").line(line_data),
        GraphType::LineFill => build::plot("").line_fill(line_data),
    };

    let plot_color = &config.graph_color;
    let plot_stroke_width = &config.graph_stroke_width;

    let header = poloto::header()
        .with_dim([plot_width, plot_height])
        .with_viewbox([plot_width, plot_height])
        .append(
            poloto::render::Theme::dark()
                .append(tagu::build::raw(format!(".poloto0.poloto_fill{{ fill: {plot_color}; }}"))) // color of first plot line filled
                .append(tagu::build::raw(format!(".poloto_line.poloto_imgs.poloto_plot{{ stroke: {plot_color}; stroke-width: {plot_stroke_width}px; }}"))) // color of first plot line
        );

    // Set plot x bounds
    let mut plot_x_bounds = vec![];
    if let Some(min_sensor_value) = config.min_sensor_value {
        plot_x_bounds.push(min_sensor_value);
    }
    if let Some(max_sensor_value) = config.max_sensor_value {
        plot_x_bounds.push(max_sensor_value);
    }

    poloto::frame_build()
        .data(poloto::plots!(line_plot, markers([], plot_x_bounds)))
        .build_and_label(("", "", ""))
        .append_to(header)
        .render_string()
        .unwrap()
}

/// Renders the given svg to an png image.
/// # Returns
/// A vector of bytes containing the RGBA8 png image
fn render_to_raster(svg_data: &str, width: u32, height: u32, background_color: &str) -> Pixmap {
    // Read our string into an SVG tree
    let usvg_tree = usvg::Tree::from_str(svg_data, &Options::default()).unwrap();

    // Extract child with id poloto_plot0
    let plot_node = usvg_tree
        .root
        .children()
        .find(|child| child.id().eq("poloto_plot0"))
        .unwrap();

    let plot_path_node = plot_node.first_child().unwrap();

    // calculate bounding box
    let bb_rect = plot_path_node.calculate_bbox().unwrap();
    let bb_height = bb_rect.height();
    let bb_width = bb_rect.width();

    // Wrap into a new root
    let new_root = usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default()));
    new_root.append(plot_path_node.clone());

    // Rendering
    let mut resvg_tree: resvg::Tree = resvg::Tree::from_usvg_node(&new_root).unwrap();
    resvg_tree.view_box.aspect.align = Align::XMinYMax;

    // Fit into the desired size
    let transform: usvg::Transform = usvg::Transform::from_translate(0f32, 0f32)
        .post_scale(width as f32 / bb_width, height as f32 / bb_height);

    // Create a new pixmap buffer to render to
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or("Pixmap allocation error")
        .unwrap();

    pixmap.fill(to_tiny_skia_color(background_color));

    // Measure the time
    resvg_tree.render(transform, &mut pixmap.as_mut());

    // Write the pixmap buffer to a PNG file
    pixmap
}

/// Convert a hex string to a tiny_skia color
/// # Arguments
/// * `hex_string` - A hex string with alpha channel like #ff0000ff
fn to_tiny_skia_color(hex_string: &str) -> tiny_skia::Color {
    let hex_string = hex_string.trim_start_matches('#');
    let hex = u32::from_str_radix(hex_string, 16).unwrap();
    let r = ((hex >> 24) & 0xff) as u8;
    let g = ((hex >> 16) & 0xff) as u8;
    let b = ((hex >> 8) & 0xff) as u8;
    let a = (hex & 0xff) as u8;
    Color::from_rgba8(r, g, b, a)
}
