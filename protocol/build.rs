use protocol_types::heros::God;
use serde_json;
use std::{fs::File, io::BufReader};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./gods.json");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let gods_rs_path = std::path::Path::new(&out_dir).join("gods.rs");
    println!(
        "{}",
        std::path::Path::new(&out_dir)
            .join("gods.rs")
            .to_str()
            .unwrap()
    );

    let file = File::open("./data/gods.json")?;
    let reader = BufReader::new(file);
    let gods: Vec<God> = serde_json::from_reader(reader)?;
    let mut gods_rs = vec![
        "use static_init::dynamic;".to_string(),
        "use protocol_types::heros::God;".to_string(),
    ];

    gods_rs.push(format!(
        "#[dynamic]\npub static GODS: Vec<God> = vec![{}];",
        gods.iter().map(|god| {
            format!(
                "God {{ id: {}, name: \"{}\".to_string(), description: \"{}\".to_string(), pantheon: \"{}\".to_string() }}",
                god.id,
                god.name,
                god.description,
                god.pantheon
            )
        }).collect::<Vec<_>>()
            .join(", ")
    ));

    std::fs::write(&gods_rs_path, gods_rs.join("\n")).unwrap();

    Ok(())
}
