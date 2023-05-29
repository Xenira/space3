use core::panic;
use image::{GenericImageView, RgbaImage};
use protocol_data_types::{CharacterJson, GodJson, Named};
use protocol_types::{character::Character, heros::God};
use serde::de::DeserializeOwned;
use serde_json;
use std::{fs::File, io::BufReader, path::Path};

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./data/gods.json");

    generate_from_json::<GodJson, God>("./data/gods", "gods", Some((256, 256)))?;
    generate_from_json::<CharacterJson, Character>(
        "./data/characters",
        "characters",
        Some((512, 512)),
    )?;

    Ok(())
}

fn generate_from_json<T, U>(
    path: &str,
    name: &str,
    image: Option<(u32, u32)>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: protocol_data_types::ToString + Named + DeserializeOwned + std::fmt::Debug,
{
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let rs_path = std::path::Path::new(&out_dir).join(format!("{}.rs", name));
    let type_name = std::any::type_name::<U>();
    let path = Path::new(path);

    let mut generated_rs = vec![
        "use std::borrow::Cow::Borrowed;".to_string(),
        "use protocol_types::prelude::*;".to_string(),
    ];

    let mut names: Vec<String> = Vec::new();

    let entities = std::fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.map(|e| e.path()).ok())
        .filter(|p| p.is_dir())
        .filter_map(|p| {
            let data_path = p.join("data.json");

            if data_path.exists() {
                File::open(data_path)
                    .ok()
                    .and_then(|f| {
                        serde_json::from_reader(BufReader::new(f))
                            .or_else(|e| {
                                warn!(
                                    "Failed to parse data.json for {:?}: {}",
                                    p.join("data.json"),
                                    e
                                );
                                Err(e)
                            })
                            .ok()
                    })
                    .map(|d: T| (p, d))
            } else {
                warn!("No data.json for {:?}", p);
                None
            }
        })
        .enumerate()
        .map(|(idx, item)| {
            if image.is_some() {
                if let Err(e) = generate_masked_img(path, &item.0, image.unwrap(), name, idx) {
                    panic!("Failed to generate masked image for {:?}: {}", item.0, e)
                }
            }

            // Check for duplicate names
            if names.contains(&item.1.get_name().to_string()) {
                panic!("Duplicate name: {}", item.1.get_name());
            }
            names.push(item.1.get_name().to_string());

            item.1.to_string(idx)
        })
        .collect::<Vec<_>>();

    if entities.is_empty() {
        warn!("No {} entries in {:?}", name, path);
    }

    generated_rs.push(format!(
        "pub static {}: [{};{}] = [{}];",
        name.to_uppercase(),
        type_name,
        entities.len(),
        entities.join(", ")
    ));

    std::fs::write(&rs_path, generated_rs.join("\n")).unwrap();

    Ok(())
}

fn generate_masked_img(
    base_path: &Path,
    path: &Path,
    image_dimensions: (u32, u32),
    name: &str,
    idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("assets")
        .join(name);
    std::fs::create_dir_all(&out_dir)?;

    let mask_img = image::open(base_path.join("mask.png"));
    let mut input_img = image::open(path.join("portrait.png")).or_else(|_| {
        warn!("No portrait for {:?}. Using fallback image.", path);
        image::open(base_path.join("fallback.png"))
    })?;

    let mut out_buffer = RgbaImage::new(image_dimensions.0, image_dimensions.1);

    if input_img.dimensions() != image_dimensions {
        input_img = input_img.resize_to_fill(
            image_dimensions.0,
            image_dimensions.1,
            image::imageops::FilterType::Nearest,
        );
    }

    if let Ok(mut mask_img) = mask_img {
        if mask_img.dimensions() != image_dimensions {
            mask_img = mask_img.resize_to_fill(
                image_dimensions.0,
                image_dimensions.1,
                image::imageops::FilterType::Nearest,
            );
        }

        for (x, y, _) in input_img.pixels() {
            let mask_pixel = mask_img.get_pixel(x, y);
            let input_pixel = input_img.get_pixel(x, y);

            let pixel_data = input_pixel
                .0
                .iter()
                .zip(mask_pixel.0)
                .map(|(i, m)| i * (m / u8::MAX))
                .collect::<Vec<_>>()[..]
                .try_into()?;
            out_buffer.put_pixel(x, y, image::Rgba(pixel_data));
        }

        out_buffer.save(out_dir.join(format!("{}.png", idx)))?;
    } else {
        input_img.save(out_dir.join(format!("{}.png", idx)))?;
    }

    Ok(())
}
