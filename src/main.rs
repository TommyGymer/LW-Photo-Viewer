#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod image_handler;

// use image::io::Reader as ImageReader;
use eframe::egui;
use eframe::epaint::Pos2;
use egui::Vec2;
use egui_extras::RetainedImage;
use image_handler::load_image_from_path;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
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

enum Direction {
    DEC,
    INC,
}

struct LwPv {
    image: RetainedImage,
    image_path: String,
    folder: String,
    press_origin: Option<Pos2>,
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
            press_origin: None,
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
            press_origin: None,
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
                image_update(self, &Direction::INC);
            }
            if ui.input().key_pressed(egui::Key::ArrowLeft) {
                image_update(self, &Direction::DEC);
            }
            if ui.input().pointer.any_down() {
                if self.press_origin == None {
                    self.press_origin = match ui.input().pointer.press_origin() {
                        Some(pos) => Some(pos),
                        None => self.press_origin,
                    };
                }
            }
            if ui.input().any_touches() {
                if self.press_origin == None {
                    self.press_origin = match ui.input().pointer.hover_pos() {
                        Some(pos) => Some(pos),
                        None => self.press_origin,
                    };
                }
            }
            if ui.input().pointer.any_released() {
                let start = self.press_origin;
                let end = match ui.input().pointer.interact_pos() {
                    Some(pos) => pos,
                    None => return
                };
                if start.unwrap().x >= end.x {
                    image_update(self, &Direction::INC);
                } else {
                    image_update(self, &Direction::DEC);
                }
                self.press_origin = None;
            }
            let scroll_delta = ui.input().scroll_delta;
            if scroll_delta.y != 0.0  {
                if scroll_delta.y > 0.0 {
                    println!("zoom in");
                } else {
                    println!("zoom out");
                }
            }
        });
    }
}

fn image_update(context: &mut LwPv, direction: &Direction) -> Option<()> {
    let files = match fs::read_dir(Path::new(&context.folder)) {
        Ok(paths) => paths,
        Err(err) => {
            println!("{:?}", err);
            return None
        },
    };

    let vec_files: Vec<fs::DirEntry> = match files.collect::<Result<Vec<_>, _>>() {
        Ok(vec_files) => vec_files,
        Err(err) => {
            println!("{:?}", err);
            return None
        },
    };

    let mut found = false;
    for i in 0..vec_files.len() {
        let index = match direction {
            Direction::INC => i,
            Direction::DEC => vec_files.len() - i - 1,
        };
        let path = vec_files[index].path();

        if found {
            context.image = match get_image(&path) {
                Some(image) => image,
                None => continue
            };
            context.image_path = path_to_string(&path)?;
            return Some(())
        } else if path.as_path() == Path::new(&context.image_path) {
            found = true;
        }
    };
    None
}

fn path_to_string(path: &PathBuf) -> Option<String> {
    Some(
        path.as_path()
            .to_str()?
            .to_string()
    )
}

fn get_image(path: &PathBuf) -> Option<RetainedImage> {
    let image = match load_image_from_path(path.as_path()) {
        Ok(image) => image,
        Err(err) => {
            println!("{:?}", err);
            return None;
        }
    };

    Some(RetainedImage::from_color_image(
        "image",
        image,
    ))
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