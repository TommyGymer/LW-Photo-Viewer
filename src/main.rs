#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// use image::io::Reader as ImageReader;
use eframe::egui;
use egui_extras::RetainedImage;
use egui::Vec2;
use image::{ImageBuffer, Rgba};
use qoi::{decode_to_buf, decode_to_vec};
use std::{fs, env};
use std::path::Path;
use std::time::Instant;
use std::io::Cursor;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

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
    let mut start = Instant::now();

    let file: &[u8] = &(fs::read(path).unwrap());

    println!("File read: {:?}", start.elapsed());
    start = Instant::now();

    let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = match path.extension() {
        Some(ext) => {
            println!("{:?}", ext.to_str().unwrap());
            match ext.to_str().unwrap() {
                "jpg" => {
                    let tmp: image::RgbaImage = turbojpeg::decompress_image(file).expect("could not read jpg");
                    tmp
                },
                "png" => {
                    let cursor = Cursor::new(file);
                    let decoder = spng::Decoder::new(cursor);
                    let (info, mut reader) = decoder.read_info().expect("could not read info");
                    let mut out: Vec<u8> = vec![0; reader.output_buffer_size()];
                    reader.next_frame(&mut out).expect("could not decode image");

                    image::ImageBuffer::from_raw(
                        info.width as u32,
                        info.height as u32,
                        out,
                    ).expect("could not find frame")
                }
                "qoi" => {
                    let (header, decoded) = decode_to_vec(file).expect("could not read qoi");
                    image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                        header.width as u32,
                        header.height as u32,
                        decoded,
                    ).expect("could not read qoi")
                }
                _ => image::load_from_memory(file)?.to_rgba8(),
            }
        },
        None => image::load_from_memory(file)?.to_rgba8(),
    };
    
    println!("Image decoded: {:?}", start.elapsed());
    start = Instant::now();

    let size = [image_buffer.width() as _, image_buffer.height() as _];

    let pixels = image_buffer.as_flat_samples();

    println!("To samples: {:?}", start.elapsed());
    start = Instant::now();

    //consumes 4 bytes at a time
    let result = Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ));

    println!("To egui image: {:?}", start.elapsed());

    result
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
                        let vec_paths: Vec<fs::DirEntry> = paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for path in vec_paths {
                            if found {
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    load_image_from_path(
                                        path.path().as_path()
                                    ).unwrap(),
                                );
                                self.image_path = path.path().as_path().to_str().unwrap().to_string();
                                break;
                            } else if path.path() == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    },
                }
            }
            if ui.input().key_pressed(egui::Key::ArrowLeft) {
                println!("left");
                match fs::read_dir("C:\\Users\\Tom\\Pictures\\写真\\") {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        let vec_paths: Vec<fs::DirEntry> = paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for i in 1..=vec_paths.len() {
                            let path = vec_paths[vec_paths.len() - i].path();
                            if found {
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    load_image_from_path(
                                        path.as_path()
                                    ).unwrap(),
                                );
                                self.image_path = path.as_path().to_str().unwrap().to_string();
                                break;
                            } else if path == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    },
                }
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