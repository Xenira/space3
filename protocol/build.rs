use core::panic;
use image::{GenericImageView, RgbaImage};
use protocol_data_types::{CharacterJson, Entity, GodJson};
use protocol_types::{character::Character, heros::God};
use quote::quote;
use quote::{format_ident, ToTokens};
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
    T: ToTokens + Entity + DeserializeOwned + std::fmt::Debug,
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
        .map(|(idx, mut item)| {
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

            item.1.with_id(idx as i32)
        })
        .collect::<Vec<_>>();

    if entities.is_empty() {
        warn!("No {} entries in {:?}", name, path);
    }

    let static_name = format_ident!("{}_ENTITIES", name.to_uppercase());
    let len = entities.len();
    let get_function = format_ident!("get_{}", name);
    let type_name = syn::parse_str::<syn::Type>(&type_name).unwrap();
    let tokens = quote! {
        use std::rc::Rc;
        use protocol_types::prelude::*;

        static mut #static_name: Option<Rc<[#type_name;#len]>> = None;

        pub fn #get_function() -> Rc<[#type_name;#len]> {
            unsafe {
                if let Some(values) = &#static_name {
                    values.clone()
                } else {
                    let values = [#(#entities),*];
                    #static_name = Some(Rc::new(values));
                    #static_name.clone().unwrap()
                }
            }
        }
    };

    std::fs::write(
        &rs_path,
        prettyplease::unparse(&syn::parse_file(tokens.to_string().as_str()).unwrap()),
    )
    .unwrap();

    warn!("Generated {:?}", rs_path);

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
