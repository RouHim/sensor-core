use criterion::{black_box, criterion_group, criterion_main, Criterion};
use font_loader::system_fonts;
use image::{ImageBuffer, Rgba};
use imageproc::drawing;
use log::error;
use sensor_core::{
    get_cache_dir, hex_to_rgba, text_renderer, ElementType, SensorType, SensorValue, TextAlign,
    TextConfig,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn criterion_benchmark(criterion: &mut Criterion) {
    std::env::set_var("SENSOR_BRIDGE_APP_NAME", "sensor-display");

    let base_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(100, 100);
    let element_id = "test";
    let text_config = TextConfig {
        sensor_id: "test".to_string(),
        font_family: "Arial".to_string(),
        font_size: 12,
        font_color: "#FFFFFFFF".to_string(),
        width: 100,
        height: 100,
        format: "{value} {unit}".to_string(),
        alignment: TextAlign::Left,
    };
    let x = 0;
    let y = 0;
    let sensor_value = SensorValue {
        id: "test".to_string(),
        value: "test".to_string(),
        unit: "test".to_string(),
        label: "".to_string(),
        sensor_type: SensorType::Text,
    };

    // Get font cache dir
    let cache_dir = get_cache_dir(element_id, &ElementType::Text).join(element_id);
    fs::remove_dir_all(cache_dir.parent().unwrap()).unwrap_or_default();
    fs::create_dir_all(cache_dir.parent().unwrap()).unwrap();

    // Write font data to it
    let font_family = system_fonts::FontPropertyBuilder::new()
        .family(&text_config.font_family)
        .build();
    let font_data = system_fonts::get(&font_family).unwrap().0;
    fs::write(cache_dir, &font_data).unwrap();
    let font = rusttype::Font::try_from_bytes(&font_data).unwrap();

    // Create a arc mutex that holds a hashmap of fonts
    let mut font_data_table = HashMap::new();
    font_data_table.insert("Arial".to_string(), font_data.clone());
    let font_data_mutex: Arc<Mutex<HashMap<String, Vec<u8>>>> =
        Arc::new(Mutex::new(font_data_table));

    // Start benchmarking
    criterion.bench_function("draw text fs", |bencher| {
        bencher.iter(|| {
            draw_text_fs(
                black_box(&mut base_image.clone()),
                black_box(element_id),
                black_box(text_config.clone()),
                black_box(x),
                black_box(y),
                black_box(Some(&sensor_value)),
            );
        })
    });

    criterion.bench_function("draw text memory", |bencher| {
        bencher.iter(|| {
            draw_text_memory(
                black_box(&mut base_image.clone()),
                black_box(&font_data_mutex),
                black_box(text_config.clone()),
                black_box(x),
                black_box(y),
                black_box(Some(&sensor_value)),
            );
        })
    });

    criterion.bench_function("draw text neo", |bencher| {
        bencher.iter(|| {
            text_renderer::render(
                black_box(base_image.width()),
                black_box(base_image.height()),
                black_box(&text_config),
                black_box(Some(&sensor_value)),
                black_box(&font),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn draw_text_fs(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    element_id: &str,
    text_config: TextConfig,
    x: i32,
    y: i32,
    sensor_value: Option<&SensorValue>,
) {
    let font_scale = rusttype::Scale::uniform(text_config.font_size as f32);
    let font_color: Rgba<u8> = hex_to_rgba(&text_config.font_color);
    let text_format = text_config.format;

    let (value, unit): (&str, &str) = match sensor_value {
        Some(sensor_value) => (&sensor_value.value, &sensor_value.unit),
        _ => ("N/A", ""),
    };

    let text = text_format
        .replace("{value}", value)
        .replace("{unit}", unit);

    let cache_dir = get_cache_dir(element_id, &ElementType::Text).join(element_id);
    let font_path = cache_dir.to_str().unwrap();

    if !Path::new(&font_path).exists() {
        error!("File {} does not exist", font_path);
        return;
    }

    let font_data = fs::read(font_path).unwrap();
    let font = rusttype::Font::try_from_bytes(&font_data).unwrap();

    drawing::draw_text_mut(image, font_color, x, y, font_scale, &font, text.as_str());
}

fn draw_text_memory(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font_data_cache: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
    text_config: TextConfig,
    x: i32,
    y: i32,
    sensor_value: Option<&SensorValue>,
) {
    let font_scale = rusttype::Scale::uniform(text_config.font_size as f32);
    let font_color: Rgba<u8> = hex_to_rgba(&text_config.font_color);
    let text_format = text_config.format;

    let (value, unit): (&str, &str) = match sensor_value {
        Some(sensor_value) => (&sensor_value.value, &sensor_value.unit),
        _ => ("N/A", ""),
    };

    let text = text_format
        .replace("{value}", value)
        .replace("{unit}", unit);

    let font_data_cache = font_data_cache.lock().unwrap();
    let font_data = font_data_cache.get(&text_config.font_family).unwrap();
    let font = rusttype::Font::try_from_bytes(font_data).unwrap();

    drawing::draw_text_mut(image, font_color, x, y, font_scale, &font, text.as_str());
}
