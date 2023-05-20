use protocol_types::{
    character::{Character, CharacterUpgrade},
    heros::God,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json;
use std::{fs::File, io::BufReader};

trait ToString {
    fn to_string(&self, idx: usize) -> String;
}

#[derive(Deserialize)]
struct GodJson {
    name: String,
    description: String,
    pantheon: String,
}

impl ToString for GodJson {
    fn to_string(&self, idx: usize) -> String {
        format!(
            "God {{ id: {}, name: std::borrow::Cow::Borrowed(\"{}\"), description: std::borrow::Cow::Borrowed(\"{}\"), pantheon: std::borrow::Cow::Borrowed(\"{}\") }}",
            idx, self.name, self.description, self.pantheon
        )
    }
}

#[derive(Deserialize)]
struct CharacterJson {
    name: String,
    description: String,
    health: i32,
    attack: i32,
    cost: i32,
    upgrade: Option<CharacterUpgrade>,
}

impl ToString for CharacterJson {
    fn to_string(&self, idx: usize) -> String {
        format!(
            "Character {{ id: {}, name: std::borrow::Cow::Borrowed(\"{}\"), description: std::borrow::Cow::Borrowed(\"{}\"), health: {}, attack: {}, cost: {}, upgrade: {}}}",
            idx,
            self.name,
            self.description,
            self.health,
            self.attack,
            self.cost,
            if let Some(upgrade) = &self.upgrade { format!("Some(protocol_types::character::CharacterUpgrade {{ name: std::borrow::Cow::Borrowed(\"{}\"), attack: {}, health: {} }})", upgrade.name, upgrade.attack, upgrade.health) } else { "None".to_string() }
        )
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./gods.json");

    generate_from_json::<GodJson, God>("./data/gods.json", "gods")?;
    generate_from_json::<CharacterJson, Character>("./data/characters.json", "characters")?;

    Ok(())
}

fn generate_from_json<T, U>(filename: &str, name: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: ToString + DeserializeOwned,
{
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let rs_path = std::path::Path::new(&out_dir).join(format!("{}.rs", name));
    let type_name = std::any::type_name::<U>();

    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let entries: Vec<T> = serde_json::from_reader(reader)?;
    let mut generated_rs = vec![format!("use {};", type_name)];

    generated_rs.push(format!(
        "pub static {}: [{};{}] = [{}];",
        name.to_uppercase(),
        type_name,
        entries.len(),
        entries
            .iter()
            .enumerate()
            .map(|(idx, item)| item.to_string(idx))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    std::fs::write(&rs_path, generated_rs.join("\n")).unwrap();

    Ok(())
}
