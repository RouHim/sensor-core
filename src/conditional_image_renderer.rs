use std::ffi::OsString;
use std::{cmp, fs};

use log::error;

use crate::{ConditionalImageConfig, ElementType, SensorType};

/// Get the image data based on the current sensor value and type
pub fn render(
    element_id: &str,
    sensor_type: &SensorType,
    conditional_image_config: &ConditionalImageConfig,
) -> Option<Vec<u8>> {
    let cache_image_folder = crate::get_cache_dir(element_id, &ElementType::ConditionalImage);
    let cache_image_folder = cache_image_folder.to_str().unwrap();

    match sensor_type {
        SensorType::Text => render_text_sensor(conditional_image_config, cache_image_folder),
        SensorType::Number => render_number_sensor(conditional_image_config, cache_image_folder),
    }
}

/// Renders a given text sensor to an conditional image
fn render_text_sensor(
    conditional_image_config: &ConditionalImageConfig,
    cache_images_folder: &str,
) -> Option<Vec<u8>> {
    // Select image based on sensor value
    let image_path = get_image_based_on_text_sensor_value(
        &conditional_image_config.sensor_value,
        cache_images_folder,
    );

    // Read image to memory
    // We heavily assume that this is already png encoded to skip the expensive png decoding
    // So just read the image here
    image_path.and_then(|image_path| fs::read(image_path).ok())
}

/// Renders a given number sensor to an conditional image
fn render_number_sensor(
    conditional_image_config: &ConditionalImageConfig,
    cache_images_folder: &str,
) -> Option<Vec<u8>> {
    // Select image based on sensor value
    let sensor_value: f64 = conditional_image_config.sensor_value.parse().unwrap();
    let image_path = get_image_based_on_numeric_sensor_value(
        conditional_image_config.min_sensor_value,
        conditional_image_config.max_sensor_value,
        sensor_value,
        cache_images_folder,
    );

    // Read image to memory
    // We heavily assume that this is already png encoded to skip the expensive png decoding
    // So just read the image here
    image_path.and_then(|image_path| fs::read(image_path).ok())
}

fn get_image_based_on_text_sensor_value(
    sensor_value: &str,
    images_folder_path: &str,
) -> Option<String> {
    let images: Vec<(String, String)> = fs::read_dir(images_folder_path)
        .unwrap()
        .flatten()
        .filter(|dir_entry| dir_entry.file_type().unwrap().is_file())
        .filter(crate::is_image)
        .map(|dir_entry| {
            (
                remove_file_extension(dir_entry.file_name()),
                dir_entry.path().to_str().unwrap().to_string(),
            )
        })
        .collect();

    // If there is no image
    if images.is_empty() {
        error!("No images found in folder {}", images_folder_path);
        return None;
    }

    // Select the image based on the lowest levehnstein distance to the sensor value
    let mut best_image_path = None;
    let mut min_distance = usize::MAX;
    for (image_name, image_path) in images {
        let distance = levenshtein_distance(sensor_value, &image_name);
        if distance < min_distance {
            min_distance = distance;
            best_image_path = Some(image_path);
        }
    }

    best_image_path
}

/// Returns the image path that fits the sensor value best
/// The sensor value is transformed to the image number coordination system
/// The image number coordination system is the range of all image numbers
/// # Arguments
/// * `sensor_min` - The minimum value of the sensor
/// * `sensor_max` - The maximum value of the sensor
/// * `sensor_value` - The current value of the sensor
/// * `images_folder` - The folder where the images are stored
fn get_image_based_on_numeric_sensor_value(
    sensor_min: f64,
    sensor_max: f64,
    sensor_value: f64,
    images_folder: &str,
) -> Option<String> {
    let numbered_images = get_image_numbers_sorted(images_folder);

    // If there is none
    if numbered_images.is_empty() {
        error!("No images found in folder {}", images_folder);
        return None;
    }

    // get min and max of images
    let image_number_min = numbered_images.first().unwrap().0 as f64;
    let image_number_max = numbered_images.last().unwrap().0 as f64;

    // Move the sensor value number into the image number coordination system / range
    let transformed_sensor_value = (sensor_value - sensor_min) / (sensor_max - sensor_min)
        * (image_number_max - image_number_min)
        + image_number_min;

    // Get the image that has the lowest distance to the calculated value
    get_best_fitting_image_path(numbered_images, transformed_sensor_value)
}

/// Returns the image name that has the lowest distance to the transformed sensor value
fn get_best_fitting_image_path(
    numbered_images: Vec<(f32, String)>,
    transformed_sensor_value: f64,
) -> Option<String> {
    let mut best_image_path = None;
    let mut min_distance = f64::MAX;
    for (number, image_path) in numbered_images {
        let distance = (transformed_sensor_value - number as f64).abs();
        if distance < min_distance {
            min_distance = distance;
            best_image_path = Some(image_path);
        }
    }
    best_image_path
}

fn remove_file_extension(file_name: OsString) -> String {
    let file_name = file_name.to_str().unwrap();
    let mut file_name = file_name.to_string();
    let extension = file_name.split('.').last();
    if let Some(extension) = extension {
        file_name = file_name
            .chars()
            .take(file_name.len() - extension.len() - 1)
            .collect();
    }
    file_name
}

/// Returns a vector of tuples with the image number and the image path
/// The vector is sorted by the image number
fn get_image_numbers_sorted(images_folder: &str) -> Vec<(f32, String)> {
    // Get all image names and parse them to numbers
    // "1.png" -> 1.0
    // "-1,123.png" -> -1.123
    let mut image_names: Vec<(f32, String)> = fs::read_dir(images_folder)
        .unwrap()
        .flatten()
        .filter(|dir_entry| dir_entry.file_type().unwrap().is_file())
        .filter(crate::is_image)
        .flat_map(|dir_entry| {
            let number = to_number(dir_entry.file_name());
            number.map(|num| (num, dir_entry.path().to_str().unwrap().to_string()))
        })
        .collect();

    // Sort by number
    image_names.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    image_names
}

/// Converts a OsString to a float number
fn to_number(string: OsString) -> Option<f32> {
    // Replace "," with "." to make it a parseable number
    let number_string = string.to_str().unwrap().replace(',', ".");

    let mut number_string: String = number_string
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
        .collect();

    // Remove all "." at the beginning
    while number_string.starts_with('.') {
        number_string = number_string.chars().skip(1).collect()
    }

    // Remove all "." at the end
    while number_string.ends_with('.') {
        number_string = number_string
            .chars()
            .take(number_string.len() - 1)
            .collect()
    }

    number_string.parse().ok()
}

/// Returns the Levenshtein distance between two strings
/// Source: https://en.wikibooks.org/wiki/Algorithm_Implementation/Strings/Levenshtein_distance#Rust
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let v1len = v1.len();
    let v2len = v2.len();

    // Early exit if one of the strings is empty
    if v1len == 0 {
        return v2len;
    }
    if v2len == 0 {
        return v1len;
    }

    fn min3<T: Ord>(v1: T, v2: T, v3: T) -> T {
        cmp::min(v1, cmp::min(v2, v3))
    }
    fn delta(x: char, y: char) -> usize {
        if x == y {
            0
        } else {
            1
        }
    }

    let mut column: Vec<usize> = (0..v1len + 1).collect();
    for x in 1..v2len + 1 {
        column[0] = x;
        let mut lastdiag = x - 1;
        for y in 1..v1len + 1 {
            let olddiag = column[y];
            column[y] = min3(
                column[y] + 1,
                column[y - 1] + 1,
                lastdiag + delta(v1[y - 1], v2[x - 1]),
            );
            lastdiag = olddiag;
        }
    }
    column[v1len]
}
