use clap::{Parser, Subcommand};
use log::info;
use onefetch_image::get_best_backend;
use protocol_data_types::{CharacterJson, GodJson};
use protocol_types::{heros::Pantheon, prelude::CharacterUpgrade};
use std::{
    error::Error,
    fs::File,
    io::{BufReader, Write},
    path::Path,
};
use surf::{Client, Config};

mod sd;

#[derive(Parser)]
#[command(author, about, version)]
struct Cli {
    #[clap(subcommand)]
    subcmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate structures", alias = "g")]
    Generate {
        #[clap(subcommand)]
        subcmd: Option<Generate>,
    },
    #[clap(about = "Migrate existing structures", alias = "m")]
    Migrate {
        #[clap(subcommand)]
        subcmd: Migrate,
    },
}

#[derive(Subcommand)]
enum Generate {
    #[clap(about = "Generate god", alias = "g")]
    God(GodInput),
}

#[derive(Subcommand)]
enum Migrate {
    #[clap(about = "Migrate god", alias = "g")]
    God,

    #[clap(about = "Migrate character", alias = "c")]
    Character,
}

#[derive(Parser)]
struct GodInput {
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long)]
    description: Option<String>,
    #[arg(short, long)]
    pantheon: Option<Pantheon>,
}

impl From<GodJson> for GodInput {
    fn from(god: GodJson) -> Self {
        Self {
            name: Some(god.name),
            description: Some(god.description),
            pantheon: Some(god.pantheon),
        }
    }
}

#[derive(Parser, Debug)]
struct CharacterInput {
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long)]
    description: Option<String>,
    #[arg(short, long)]
    health: Option<u32>,
    #[arg(short, long)]
    attack: Option<u32>,
    #[arg(short, long)]
    cost: Option<u32>,
}

impl From<CharacterJson> for CharacterInput {
    fn from(character: CharacterJson) -> Self {
        Self {
            name: Some(character.name),
            description: Some(character.description),
            health: Some(character.health as u32),
            attack: Some(character.attack as u32),
            cost: Some(character.cost as u32),
        }
    }
}

static mut client: Option<Client> = None;

#[tokio::main]
async fn main() {
    unsafe {
        client = Config::new().set_timeout(None).try_into().ok();
    }

    let args = Cli::parse();

    match args.subcmd {
        Commands::Generate { subcmd } => {
            if let Some(subcmd) = subcmd {
                match subcmd {
                    Generate::God(god) => {
                        generate_god(god).await.unwrap();
                    }
                }
            }
        }
        Commands::Migrate { subcmd } => match subcmd {
            Migrate::God => {
                let gods: Vec<GodJson> = serde_json::from_reader(BufReader::new(
                    File::open("../protocol/data/gods.json").unwrap(),
                ))
                .unwrap();
                for god in gods {
                    generate_god(god.into()).await.unwrap();
                }
            }
            Migrate::Character => {
                let characters: Vec<CharacterJson> = serde_json::from_reader(BufReader::new(
                    File::open("../protocol/data/characters.json").unwrap(),
                ))
                .unwrap();
                for character in characters {
                    generate_character(character.into()).await.unwrap();
                }
            }
        },
    }
}

async fn generate_god(god: GodInput) -> Result<(), Box<dyn Error>> {
    let god_name = god.name.clone();
    let existing: Option<GodJson> = if god_name.is_some() {
        serde_json::from_reader(BufReader::new(
            File::open(
                Path::new("../protocol/data/gods/")
                    .join(god_name.unwrap().to_lowercase())
                    .join("data.json"),
            )
            .unwrap(),
        ))
        .unwrap()
    } else {
        None
    };

    let god = GodJson {
        id: None,
        name: titlecase(&mut god.name.ok_or(()).or_else(|_| {
            inquire::Text::new("God name")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.name)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
        })?),
        description: god.description.ok_or(()).or_else(|_| {
            inquire::Text::new("God description:")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.description)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
        })?,
        pantheon: god.pantheon.ok_or(()).or_else(|_| {
            inquire::Select::new("God pantheon:", Pantheon::VARIANTS.to_vec())
                .with_help_message(
                    existing
                        .map(|e| e.pantheon.to_string())
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
        })?,
    };
    let god_json = serde_json::to_string_pretty(&god)?;

    println!("{}", &god_json);

    let god_dir = Path::new("../protocol/data/gods").join(god.name.to_lowercase());
    if inquire::Confirm::new("Save god?")
        .with_default(true)
        .prompt()?
    {
        std::fs::create_dir_all(&god_dir)?;
        File::create(god_dir.join("data.json"))?.write_all(god_json.as_bytes())?;
    }

    if !god_dir.join("portrait.png").exists()
        || inquire::Confirm::new("Overwrite portrait?")
            .with_default(false)
            .prompt()?
    {
        let gender = inquire::Select::new("God/Goddess:", vec!["God", "Goddess"]).prompt()?;
        let element = inquire::Text::new("Element:").prompt()?;
        prompt_image(
            &format!(
                "{} {} {}, {}, dynamic pose",
                god.pantheon, gender, god.name, element
            ),
            &god_dir,
        )
        .await?;
    }

    Ok(())
}

async fn generate_character(character: CharacterInput) -> Result<(), Box<dyn Error>> {
    println!("{:?}", character);
    let character_name = character.name.clone();
    let existing: Option<CharacterJson> = if character_name.is_some() {
        let character_path = Path::new("../protocol/data/characters")
            .join(character_name.clone().unwrap().to_lowercase());
        if character_path.exists() {
            serde_json::from_reader(BufReader::new(
                File::open(
                    Path::new("../protocol/data/characters/")
                        .join(character_name.unwrap().to_lowercase())
                        .join("data.json"),
                )
                .unwrap(),
            ))
            .map(|c| {
                println!("{:?}", c);
                c
            })
            .unwrap()
        } else {
            None
        }
    } else {
        None
    };

    let character = CharacterJson {
        id: None,
        name: titlecase(&mut character.name.clone().ok_or(()).or_else(|_| {
            inquire::Text::new("Character name")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.name)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
        })?),
        description: character.description.ok_or(()).or_else(|_| {
            inquire::Text::new("Character description:")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.description)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
        })?,
        attack: character.attack.ok_or(()).map(|a| a as i32).or_else(|_| {
            inquire::Text::new("Character description:")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.description)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
                .unwrap()
                .parse()
        })?,
        health: character.health.ok_or(()).map(|a| a as i32).or_else(|_| {
            inquire::Text::new("Character health:")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.description)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
                .unwrap()
                .parse()
        })?,
        cost: character.cost.ok_or(()).map(|a| a as u8).or_else(|_| {
            inquire::Text::new("Character cost:")
                .with_default(
                    existing
                        .clone()
                        .map(|e| e.description)
                        .unwrap_or("".to_string())
                        .as_str(),
                )
                .prompt()
                .unwrap()
                .parse()
        })?,
        upgrade: if inquire::Confirm::new("Character upgradable?")
            .with_default(
                existing
                    .clone()
                    .map(|e| e.upgrade.is_some())
                    .unwrap_or_else(|| true),
            )
            .prompt()?
        {
            Some(CharacterUpgrade {
                name: titlecase(
                    &mut character
                        .name
                        .as_ref()
                        .ok_or(())
                        .map(|a| a.to_string())
                        .or_else(|_| {
                            inquire::Text::new("Upgraded Character name")
                                .with_default(
                                    &existing
                                        .clone()
                                        .map(|e| {
                                            e.upgrade.map(|u| u.name).unwrap_or("".to_string())
                                        })
                                        .unwrap_or("".to_string()),
                                )
                                .prompt()
                        })
                        .unwrap(),
                ),
                attack: character
                    .attack
                    .ok_or(())
                    .map(|a| a.to_string())
                    .or_else(|_| {
                        inquire::Text::new("Upgraded Character attack:")
                            .with_default(
                                &existing
                                    .clone()
                                    .map(|e| {
                                        e.upgrade
                                            .map(|u| u.attack.to_string())
                                            .unwrap_or(character.attack.unwrap_or(0).to_string())
                                    })
                                    .unwrap_or((character.attack.unwrap_or(0) * 2).to_string()),
                            )
                            .prompt()
                    })
                    .unwrap()
                    .parse()
                    .unwrap(),
                health: character
                    .health
                    .ok_or(())
                    .map(|a| a.to_string())
                    .or_else(|_| {
                        inquire::Text::new("Upgraded Character health:")
                            .with_default(
                                &existing
                                    .clone()
                                    .map(|e| {
                                        e.upgrade
                                            .map(|u| u.attack.to_string())
                                            .unwrap_or(character.attack.unwrap_or(0).to_string())
                                    })
                                    .unwrap_or((character.attack.unwrap_or(0) * 2).to_string()),
                            )
                            .prompt()
                    })
                    .unwrap()
                    .parse()
                    .unwrap(),
                abilities: vec![], // TODO
            })
        } else {
            None
        },
        abilities: vec![], // TODO
    };
    let character_json = serde_json::to_string_pretty(&character)?;

    println!("{}", &character_json);

    let character_dir =
        Path::new("../protocol/data/characters").join(character.name.to_lowercase());
    if inquire::Confirm::new("Save character?")
        .with_default(true)
        .prompt()?
    {
        std::fs::create_dir_all(&character_dir)?;
        File::create(character_dir.join("data.json"))?.write_all(character_json.as_bytes())?;
    }

    if !character_dir.join("portrait.png").exists()
        || inquire::Confirm::new("Overwrite portrait?")
            .with_default(false)
            .prompt()?
    {
        let element = inquire::Text::new("Prompt:").prompt()?;
        prompt_image(
            &format!("{}, {}, dynamic pose", character.name, element),
            &character_dir,
        )
        .await?;
    }

    Ok(())
}

fn titlecase(s: &mut str) -> String {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s.to_string()
}

async fn prompt_image(prompt: &str, out: &Path) -> Result<(), Box<dyn Error>> {
    let mut prompt = inquire::Text::new("Prompt:")
        .with_default(prompt)
        .prompt()?;
    loop {
        info!("Generating images...");
        sd::get_image(unsafe { client.as_ref().unwrap() }, &prompt).await;

        log_image_to_console("image_0.png", 25);

        let selection = inquire::Select::new(
            "Select image:",
            vec!["1", "2", "3", "4", "Retry", "Edit Prompt", "Cancel"],
        )
        .prompt()?;
        match selection {
            "1" | "2" | "3" | "4" => {
                std::fs::copy(format!("image_{}.png", selection), out.join("portrait.png"))?;
                std::fs::copy("info.txt", out.join("info.txt"))?;
                break;
            }
            "Retry" => {
                continue;
            }
            "Edit Prompt" => {
                prompt = inquire::Text::new("Prompt:")
                    .with_default(&prompt)
                    .prompt()?;
            }
            "Cancel" => {
                return Ok(());
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn log_image_to_console(image: &str, lines: usize) {
    let lines = vec![""]
        .repeat(lines)
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    if let Some(backend) = get_best_backend() {
        print!(
            "{}",
            backend
                .add_image(lines, &image::open(image).unwrap(), 16)
                .unwrap(),
        );
    } else {
        println!("No backend found");
    }
}
