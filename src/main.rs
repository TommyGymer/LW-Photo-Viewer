#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// use image::io::Reader as ImageReader;
use eframe::egui;
use eframe::epaint::Color32;
use egui_extras::RetainedImage;
use egui::Vec2;
use image::error::{DecodingError, ImageFormatHint};
use image::{Rgba, Rgb};
use qoi::decode_to_vec;
use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSlice;
use std::ffi::OsString;
use std::{fs, env};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::io::Cursor;
use windows::Win32::UI::Shell::IThumbnailCache;
use windows::Win32::UI::Shell::PropertiesSystem::IInitializeWithStream;
use notify_rust::Notification;

//https://learn.microsoft.com/en-gb/samples/microsoft/windows-classic-samples/recipethumbnailprovider/

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    dbg!(&args);

    // IInitializeWithStream::Initialize(&self, pstream, grfmode);

    let mut content = String::from("");
    args.iter()
        .for_each(|i| {
            content.push_str(i.to_str().unwrap_or("unknown char string"));
            content.push_str(" ");
    });

    Notification::new()
        .summary("LW Photoviewer")
        .body(&content)
        .show().unwrap();

    let path: String = if args.len() == 1 {
        String::from("./img/no_image.png")
    } else {
        String::from(args[1].to_str().unwrap())
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

    // unsafe {
    //     IThumbnailCache::GetThumbnail(&self, pshellitem, cxyrequestedthumbsize, flags, ppvthumb, poutflags, pthumbnailid);
    // }
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
            image_path: String::from(
                "./img/no_image.png"
            ),
            folder: String::from(
                "./img/"
            ),
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

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let start = Instant::now();
    println!("{:?}", path.as_os_str());
    let file: &[u8] = &(fs::read(path).unwrap());

    let result = match path.extension() {
        Some(ext) => {
            match ext.to_str().unwrap() {
                "jpg" => {
                    let tmp: image::RgbImage = decode_jpg(file, path.to_path_buf())?;
                    Ok(color_image_from_rgb_image_buffer(tmp)?)
                },
                "jfif" => {
                    let tmp: image::RgbImage = decode_jpg(file, path.to_path_buf())?;
                    Ok(color_image_from_rgb_image_buffer(tmp)?)
                }
                "png" => {
                    let cursor = Cursor::new(file);
                    let decoder = spng::Decoder::new(cursor);
                    let (info, mut reader) = decoder.read_info().expect("could not read info");
                    
                    let mut out: Vec<u8> = vec![0; reader.output_buffer_size()];
                    reader.next_frame(&mut out).unwrap();
                    
                    match info.color_type {
                        spng::ColorType::Truecolor => {
                            color_image_from_rgb_image_buffer(
                                rgb_image_from_raw(
                                    info.width,
                                    info.height,
                                    out,
                                    path.to_path_buf(),
                                )?
                            )
                        },
                        spng::ColorType::TruecolorAlpha => {
                            color_image_from_rgba_image_buffer(
                                rgba_image_from_raw(
                                    info.width,
                                    info.height,
                                    out,
                                    path.to_path_buf(),
                                )?
                            )
                        },
                        _ => panic!("not implemented grayscale png"),
                    }
                }
                "qoi" => {
                    let (header, decoded) = decode_to_vec(file).expect("could not read qoi");
                    match header.channels {
                        qoi::Channels::Rgb => {
                            color_image_from_rgb_image_buffer(
                                rgb_image_from_raw(
                                    header.width,
                                    header.height,
                                    decoded,
                                    path.to_path_buf(),
                                )?
                            )
                        },
                        qoi::Channels::Rgba => {
                            color_image_from_rgba_image_buffer(
                                rgba_image_from_raw(
                                    header.width,
                                    header.height,
                                    decoded,
                                    path.to_path_buf(),
                                )?
                            )
                        },
                    }
                }
                _ => color_image_from_rgba_image_buffer(
                        image::load_from_memory(file)?
                            .to_rgba8()
                    ),
            }
        },
        None => Ok(
            color_image_from_rgba_image_buffer(
                image::load_from_memory(file)?
                    .to_rgba8()
            )?
        ),
    };
    println!("Total: {:?}", start.elapsed());
    result
}

fn decode_jpg(file: &[u8], ext: PathBuf) -> Result<image::RgbImage, image::ImageError> {
    match turbojpeg::decompress_image(file) {
        Ok(tmp) => Ok(tmp),
        Err(err) => Err(image::ImageError::Decoding(DecodingError::new(ImageFormatHint::PathExtension(ext), err))),
    }
}

fn rgb_image_from_raw(width: u32, height: u32, data: Vec<u8>, ext: PathBuf) -> Result<image::RgbImage, image::ImageError> {
    let start = Instant::now();
    let result = match image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        width,
        height,
        data,
    ) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(ImageFormatHint::PathExtension(ext), "Raw image bytes did not fit the image container"))),
    };
    println!("RGB image from raw: {:?}", start.elapsed());
    result
}

fn rgba_image_from_raw(width: u32, height: u32, data: Vec<u8>, ext: PathBuf) -> Result<image::RgbaImage, image::ImageError> {
    let start = Instant::now();
    let result = match image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        width,
        height,
        data,
    ) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(ImageFormatHint::PathExtension(ext), "Raw image bytes did not fit the image container"))),
    };
    println!("RGBA image from raw: {:?}", start.elapsed());
    result
}

fn from_rgb_unmultiplied(size: [usize; 2], rgb: &[u8]) -> egui::ColorImage {
    //TODO: try Vec.concat with larger consume sizes
    assert_eq!(size[0] * size[1] * 3, rgb.len());
    let pixels = rgb
        .par_chunks_exact(3)
        .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
        .collect();
    egui::ColorImage { size, pixels }
}

fn from_rgba_unmultiplied(size: [usize; 2], rgba: &[u8]) -> egui::ColorImage {
    //TODO: try Vec.concat with larger consume sizes
    assert_eq!(size[0] * size[1] * 4, rgba.len());
    let pixels = rgba
        .par_chunks_exact(4)
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    egui::ColorImage { size, pixels }
}

fn color_image_from_rgb_image_buffer(image: image::RgbImage) -> Result<egui::ColorImage, image::ImageError>{
    let start = Instant::now();
    let size = [image.width() as _, image.height() as _];
    let pixels = image.as_flat_samples();
    let result = Ok(from_rgb_unmultiplied(
        size,
        pixels.as_slice(),
    ));
    println!("egui RGB ColorImage: {:?}", start.elapsed());
    result
}

fn color_image_from_rgba_image_buffer(image: image::RgbaImage) -> Result<egui::ColorImage, image::ImageError>{
    let start = Instant::now();
    let size = [image.width() as _, image.height() as _];
    let pixels = image.as_flat_samples();
    let result = Ok(from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ));
    println!("egui RGBA ColorImage: {:?}", start.elapsed());
    result
}

impl eframe::App for LwPv {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        _frame.set_window_title(&self.image_path);
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
                match fs::read_dir(Path::new(&self.folder)) {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        let vec_paths: Vec<fs::DirEntry> = paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for path in vec_paths {
                            if found {
                                self.image_path = path.path().as_path().to_str().unwrap().to_string();
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    match load_image_from_path(
                                        path.path().as_path()
                                    ) {
                                        Ok(image) => image,
                                        Err(err) => {
                                            println!("{:?}", err);
                                            break;
                                        },
                                    },
                                );
                                break;
                            } else if path.path().as_path() == Path::new(&self.image_path) {
                                found = true;
                            }
                        }
                    },
                }
            }
            if ui.input().key_pressed(egui::Key::ArrowLeft) {
                match fs::read_dir(Path::new(&self.folder)) {
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => {
                        let mut found = false;
                        let vec_paths: Vec<fs::DirEntry> = paths.collect::<Result<Vec<_>, _>>().unwrap();
                        for i in 1..=vec_paths.len() {
                            let path = vec_paths[vec_paths.len() - i].path();
                            if found {
                                self.image_path = path.as_path().to_str().unwrap().to_string();
                                self.image = RetainedImage::from_color_image(
                                    "image",
                                    match load_image_from_path(
                                        path.as_path()
                                    ) {
                                        Ok(image) => image,
                                        Err(err) => {
                                            println!("{:?}", err);
                                            break;
                                        },
                                    },
                                );
                                break;
                            } else if path.as_path() == Path::new(&self.image_path) {
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