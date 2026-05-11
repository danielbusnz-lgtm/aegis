use image::image_dimensions;
use image::imageops::FilterType;
use std::process::Command;

pub fn take_area_screenshot(x: i32, y: i32, width: i32, height: i32, filename: &str) {
    let geometry = format!("{},{} {}x{}", x, y, width, height);

    let status = Command::new("grim")
        .args(["-g", &geometry, filename])
        .status()
        .expect("Failed to execute grim");

    if status.success() {
        resize_screenshot(filename, 1920);
        println!("Screenshot saved to {}", filename);
    } else {
        eprintln!("Error taking screenshot");
    }
}

pub fn resize_screenshot(filename: &str, max_long_edge: u32) {
    let (w, h) = match image_dimensions(filename) {
        Ok(dim) => dim,
        Err(e) => {
            eprintln!("Error reading image dimensions: {}", e);
            return;
        }
    };

    if w.max(h) <= max_long_edge {
        return;
    }

    let img = match image::open(filename) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error opening image: {}", e);
            return;
        }
    };

    let resized = img.resize(max_long_edge, max_long_edge, FilterType::Lanczos3);

    if let Err(e) = resized.save(filename) {
        eprintln!("Error saving resized image: {}", e);
    }
}
