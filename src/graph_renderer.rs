use std::rc::Rc;

use crate::{GraphConfig, GraphType};
use poloto::build;
use resvg::tiny_skia::{Color, Rect};
use resvg::usvg::{Align, NodeExt, Options, PaintOrder, TreeParsing};
use resvg::{tiny_skia, usvg};

use tagu::prelude::Elem;

/// Renders a graph based on the given config
/// # Returns
/// A vector of bytes containing the RGB8 png image
/// # Arguments
/// * `graph_config` - The config for the graph
pub fn render(graph_config: GraphConfig) -> Vec<u8> {
    let width = graph_config.width;
    let height = graph_config.height;

    let plot_data = prepare_plot_data(width, graph_config.sensor_values);

    let svg_data = render_graph(
        plot_data,
        height as f64,
        width as f64,
        graph_config.graph_type,
        &graph_config.graph_color,
        graph_config.graph_stroke_width,
    );

    render_to_png(
        &svg_data,
        width,
        height,
        &graph_config.background_color,
        graph_config.border_color,
    )
}

/// Prepares the plot data for the graph.
/// Aligns the sensor values to the width of the desired graph width.
fn prepare_plot_data(width: u32, sensor_values: Vec<f64>) -> Vec<f64> {
    // Ensure that sensor values does not exceed the width, if so cut them and keep the last values
    let sensor_values = if sensor_values.len() > width as usize {
        sensor_values[(sensor_values.len() - width as usize)..].to_vec()
    } else {
        sensor_values
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
fn render_graph(
    some_numbers: Vec<f64>,
    image_render_height: f64,
    image_render_width: f64,
    plot_type: GraphType,
    plot_color: &str,
    plot_stroke_width: i32,
) -> String {
    // Because we are going to extract only the path from the svg,
    // we should render the total plot greater than the actual desired image
    // So we get a decent quality
    let plot_width = image_render_height * 2.0;
    let plot_height = image_render_width * 2.0;

    let line_data: Vec<[f64; 2]> = some_numbers
        .iter()
        .enumerate()
        .map(|(i, x)| [i as f64, *x])
        .collect();

    let line_plot = match plot_type {
        GraphType::Line => build::plot("").line(line_data),
        GraphType::LineFill => build::plot("").line_fill(line_data),
    };

    let header = poloto::header()
        .with_dim([plot_width, plot_height])
        .with_viewbox([plot_width, plot_height])
        .append(
            poloto::render::Theme::dark()
                .append(tagu::build::raw(format!(".poloto0.poloto_fill{{ fill: {plot_color}; }}"))) // color of first plot line filled
                .append(tagu::build::raw(format!(".poloto_line.poloto_imgs.poloto_plot{{ stroke: {plot_color}; stroke-width: {plot_stroke_width}px; }}"))) // color of first plot line
        );

    poloto::frame_build()
        .data(poloto::plots!(line_plot))
        .build_and_label(("", "", ""))
        .append_to(header)
        .render_string()
        .unwrap()
}

/// Renders the given svg to an png image.
/// # Returns
/// A vector of bytes containing the RGBA8 png image
fn render_to_png(
    svg_data: &str,
    width: u32,
    height: u32,
    background_color: &str,
    border_color: String,
) -> Vec<u8> {
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

    // Draw a 1px border in red around use bounding box of path element
    new_root.append(create_border(&border_color, bb_rect));

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
    pixmap.encode_png().unwrap()
}

/// Create a border around the line chart
/// The border is drawn around the bounding box of the line chart
/// and moved one pixel to the inner side
/// This is done to avoid that the border is cut off
fn create_border(border_color: &str, line_chart_bounding_box: Rect) -> usvg::Node {
    // Draw border around line chart and move one pixel to the inner side
    let rect = Rect::from_ltrb(
        line_chart_bounding_box.left() + 1.0,
        line_chart_bounding_box.top() + 1.0,
        line_chart_bounding_box.right() - 1.0,
        line_chart_bounding_box.bottom() - 1.0,
    )
    .unwrap();

    let stroke_path = tiny_skia::PathBuilder::from_rect(rect)
        .stroke(&tiny_skia::Stroke::default(), 1.0)
        .unwrap();

    // Get and remove the alpha value from border color
    let alpha_hex = border_color.get(7..9).unwrap();
    let border_color = border_color.get(0..7).unwrap();

    // Convert hex color string to usvg Fill
    let mut paint = usvg::Fill::from_paint(usvg::Paint::Color(to_usvg_color(border_color)));

    // Convert alpha hex value to a value between 0 and 1
    let alpha_float = u8::from_str_radix(alpha_hex, 16).unwrap() as f64 / 255.0;

    // if alpha is 0, we don't need to render it
    if alpha_float.eq(&0.0) {
        return usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default()));
    }

    paint.opacity = usvg::Opacity::new_clamped(alpha_float as f32);

    usvg::Node::new(usvg::NodeKind::Path(usvg::Path {
        id: "border".to_string(),
        transform: Default::default(),
        visibility: Default::default(),
        fill: Some(paint),
        stroke: None,
        paint_order: PaintOrder::StrokeAndFill,
        rendering_mode: Default::default(),
        text_bbox: None,
        data: Rc::from(stroke_path),
    }))
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

/// Convert a hex string to a usvg color
/// # Arguments
/// * `hex_string` - A hex string like #ff0000 WITHOUT alpha channel
fn to_usvg_color(hex_string: &str) -> usvg::Color {
    let hex_string = hex_string.trim_start_matches('#');
    let hex = u32::from_str_radix(hex_string, 16).unwrap();
    let r = ((hex >> 16) & 0xff) as u8;
    let g = ((hex >> 8) & 0xff) as u8;
    let b = (hex & 0xff) as u8;
    usvg::Color::new_rgb(r, g, b)
}
