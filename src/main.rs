#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod image_handler;

// use image::io::Reader as ImageReader;
use eframe::egui;
use egui::Vec2;
use egui_extras::RetainedImage;
use image_handler::load_image_from_path;
use std::ffi::OsString;
use std::path::Path;
use std::{env, fs};

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    dbg!(&args);

    // IInitializeWithStream::Initialize(&self, pstream, grfmode);
    // IThumbnailCache::GetThumbnail(&self, pshellitem, cxyrequestedthumbsize, flags, ppvthumb, poutflags, pthumbnailid);

    let path: String = if args.len() == 1 {
        String::from("./img/no_image.png")
    } else {
        String::from(args[1].to_str().unwrap_or("./img/no_image.png"))
    };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "LW Photo Viewer",
        options,
        Box::new(|_cc| Box::new(LwPv::set_paths(path))),
    );
}

struct LwPv {
    image: RetainedImage,
    image_path: String,
    folder: String,
}

impl Default for LwPv {
    fn default() -> Self {
        Self {
            image: RetainedImage::from_color_image(
                "image",
                load_image_from_path(Path::new("./img/no_image.png")).unwrap(),
            ),
            image_path: String::from("./img/no_image.png"),
            folder: String::from("./img/"),
        }
    }
}

impl LwPv {
    fn set_paths(path: String) -> Self {
        let parent = String::from(Path::new(&path).parent().unwrap().to_str().unwrap());
        Self {
            image: RetainedImage::from_color_image(
                "image",
                load_image_from_path(Path::new(&path)).unwrap(),
            ),
            image_path: path,
            folder: parent,
        }
    }
}



impl eframe::App for LwPv {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        _frame.set_window_title(&self.image_path);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Image::new(
                    self.image.texture_id(ctx),
                    max_maintain_ar(ui.available_size(), self.image.size_vec2()),
                ));
            });
            if ui.input().key_pressed(egui::Key::ArrowRight) {
                match fs::read_dir(Path::new(&self.folder)) {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        let vec_paths: Vec<fs::DirEntry> =
                            paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for path in vec_paths {
                            if found {
                                self.image_path = path
                                    .path()
                                    .as_path()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    match load_image_from_path(path.path().as_path()) {
                                        Ok(image) => image,
                                        Err(err) => {
                                            println!("{:?}", err);
                                            break;
                                        }
                                    },
                                );
                                break;
                            } else if path.path().as_path() == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    }
                }
            }
            if ui.input().key_pressed(egui::Key::ArrowLeft) {
                match fs::read_dir(Path::new(&self.folder)) {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        let vec_paths: Vec<fs::DirEntry> =
                            paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for i in 1..=vec_paths.len() {
                            let path = vec_paths[vec_paths.len() - i].path();
                            if found {
                                self.image_path = path
                                    .as_path()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    match load_image_from_path(path.as_path()) {
                                        Ok(image) => image,
                                        Err(err) => {
                                            println!("{:?}", err);
                                            break;
                                        }
                                    },
                                );
                                break;
                            } else if path.as_path() == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    }
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