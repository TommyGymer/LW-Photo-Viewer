#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// use image::io::Reader as ImageReader;
use eframe::egui;
use egui_extras::RetainedImage;
use egui::ColorImage;
use egui::Vec2;
use std::fs;
use std::path::Path;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 900.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|_cc| Box::new(LwPv::default())),
    );
}

struct LwPv {
    image: RetainedImage,
    image_path: String,
}

impl Default for LwPv {
    fn default() -> Self {
        Self {
            image: RetainedImage::from_color_image(
                "image",
                load_image_from_path(Path::new("C:\\Users\\Tom\\Pictures\\写真\\1116473.jpg")).unwrap(),
            ),
            image_path: String::from(
                "C:\\Users\\Tom\\Pictures\\写真\\1116473.jpg"
            ),
        }
    }
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl eframe::App for LwPv {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(
                    egui::Image::new(
                        self.image.texture_id(ctx),
                        max_maintain_ar(ui.available_size(), self.image.size_vec2()),
                    )
                );
            });
            if ui.input().key_pressed(egui::Key::ArrowRight) {
                println!("right");
                match fs::read_dir("C:\\Users\\Tom\\Pictures\\写真\\") {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        for path in paths {
                            if found {
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    load_image_from_path(
                                        path.unwrap().path().as_path()
                                    ).unwrap(),
                                );
                                self.image_path = path.unwrap().path().as_path().to_str().unwrap().to_string();
                                break;
                            } else if path.unwrap().path() == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    },
                }
            }
            if ui.input().key_pressed(egui::Key::ArrowLeft) {
                println!("left");
            }
        });
    }
}

fn max_maintain_ar(available: Vec2, image: Vec2) -> Vec2 {
    let frame_ar = available.y / available.x;
    let image_ar = image.y / image.x;
    if frame_ar > image_ar {
        egui::Vec2::new(available.x, available.x * image_ar)
    } else {
        egui::Vec2::new(available.y / image_ar, available.y)
    }
}