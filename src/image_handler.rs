pub use eframe::epaint::Color32;
pub use image::error::{DecodingError, ImageFormatHint};
pub use image::{Rgb, Rgba};
pub use qoi::decode_to_vec;
pub use rayon::iter::ParallelIterator;
pub use rayon::slice::ParallelSlice;
pub use std::time::Instant;
pub use std::io::Cursor;
pub use std::path::{Path, PathBuf};
use std::fs;
use eframe::egui;

pub fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let start = Instant::now();
    println!("{:?}", path.as_os_str());
    let file: &[u8] = &(fs::read(path).unwrap());

    let result = match path.extension() {
        Some(ext) => match ext.to_str().unwrap() {
            "jpg" => Ok(color_image_from_rgb_image_buffer(decode_jpg(
                file,
                path.to_path_buf(),
            )?)?),
            "jfif" => Ok(color_image_from_rgb_image_buffer(decode_jpg(
                file,
                path.to_path_buf(),
            )?)?),
            "png" => {
                let png_start = Instant::now();
                let cursor = Cursor::new(file);
                let decoder = spng::Decoder::new(cursor);
                let (info, mut reader) = decoder.read_info().expect("could not read info");

                let mut out: Vec<u8> = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut out).unwrap();
                println!("PNG decoded: {:?}", png_start.elapsed());

                match info.color_type {
                    spng::ColorType::Truecolor => color_image_from_rgb_image_buffer(
                        rgb_image_from_raw(info.width, info.height, out, path.to_path_buf())?,
                    ),
                    spng::ColorType::TruecolorAlpha => color_image_from_rgba_image_buffer(
                        rgba_image_from_raw(info.width, info.height, out, path.to_path_buf())?,
                    ),
                    _ => panic!("not implemented grayscale png; this should probably return an error"),
                }
            }
            "qoi" => {
                let qoi_start = Instant::now();
                let (header, decoded) = decode_to_vec(file).expect("could not read qoi");
                println!("QOI decoded: {:?}", qoi_start.elapsed());
                match header.channels {
                    qoi::Channels::Rgb => {
                        let raw = rgb_image_from_raw(
                            header.width,
                            header.height,
                            decoded,
                            path.to_path_buf(),
                        )?;
                        color_image_from_rgb_image_buffer(raw)
                    },
                    qoi::Channels::Rgba => {
                        let raw = rgba_image_from_raw(
                            header.width,
                            header.height,
                            decoded,
                            path.to_path_buf(),
                        )?;
                        color_image_from_rgba_image_buffer(raw)
                    },
                }
            }
            _ => color_image_from_rgba_image_buffer(image::load_from_memory(file)?.to_rgba8()),
        },
        None => Ok(color_image_from_rgba_image_buffer(
            image::load_from_memory(file)?.to_rgba8(),
        )?),
    };

    println!("Total: {:?}", start.elapsed());
    result
}

pub fn decode_jpg(file: &[u8], ext: PathBuf) -> Result<image::RgbImage, image::ImageError> {
    match turbojpeg::decompress_image(file) {
        Ok(tmp) => Ok(tmp),
        Err(err) => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            err,
        ))),
    }
}

pub fn rgb_image_from_raw(
    width: u32,
    height: u32,
    data: Vec<u8>,
    ext: PathBuf,
) -> Result<image::RgbImage, image::ImageError> {
    match image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, data) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            "Raw image bytes did not fit the image container",
        ))),
    }
}

pub fn rgba_image_from_raw(
    width: u32,
    height: u32,
    data: Vec<u8>,
    ext: PathBuf,
) -> Result<image::RgbaImage, image::ImageError> {
    match image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, data) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            "Raw image bytes did not fit the image container",
        ))),
    }
}

pub fn from_rgb_unmultiplied(size: [usize; 2], rgb: &[u8]) -> egui::ColorImage {
    //TODO: try Vec.concat with larger consume sizes
    assert_eq!(size[0] * size[1] * 3, rgb.len());
    let pixels = rgb
        .par_chunks_exact(3)
        .map(|p| Color32::from_rgb(p[0], p[1], p[2]))
        .collect();
    egui::ColorImage { size, pixels }
}

pub fn from_rgba_unmultiplied(size: [usize; 2], rgba: &[u8]) -> egui::ColorImage {
    //TODO: try Vec.concat with larger consume sizes
    assert_eq!(size[0] * size[1] * 4, rgba.len());
    let pixels = rgba
        .par_chunks_exact(4)
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    egui::ColorImage { size, pixels }
}

pub fn color_image_from_rgb_image_buffer(
    image: image::RgbImage,
) -> Result<egui::ColorImage, image::ImageError> {
    let start = Instant::now();
    let size = [image.width() as _, image.height() as _];
    let pixels = image.as_flat_samples();
    let result = Ok(from_rgb_unmultiplied(size, pixels.as_slice()));
    println!("ColorImage from RGB image: {:?}", start.elapsed());
    result
}

pub fn color_image_from_rgba_image_buffer(
    image: image::RgbaImage,
) -> Result<egui::ColorImage, image::ImageError> {
    let start = Instant::now();
    let size = [image.width() as _, image.height() as _];
    let pixels = image.as_flat_samples();
    let result = Ok(from_rgba_unmultiplied(size, pixels.as_slice()));
    println!("ColorImage from RGBA image: {:?}", start.elapsed());
    result
}