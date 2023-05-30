use std::fmt::{format, Debug};

use protocol_types::{
    character::CharacterUpgrade,
    heros::Pantheon,
    prelude::{Ability, AbilityEffect, AbilityValue},
};
use quote::{quote, ToTokens, TokenStreamExt, __private::TokenStream, format_ident, quote_spanned};
use serde::{Deserialize, Serialize};
use std::borrow::Cow::Borrowed;

pub trait Entity {
    fn with_id(&mut self, id: i32) -> Self;
    fn get_name(&self) -> &str;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GodJson {
    pub id: Option<i32>,
    pub name: String,
    pub description: String,
    pub pantheon: Pantheon,
}

impl ToTokens for GodJson {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let id = self.id.unwrap_or(0);
        let name = Borrowed(&self.name);
        let description = Borrowed(&self.description);
        let pantheon = format_ident!("{}", format!("{:?}", self.pantheon));
        tokens.extend(quote! {
            God {
                id: #id,
                name: #name.to_string(),
                description: #description.to_string(),
                pantheon: Pantheon::#pantheon,
            }
        });
    }
}

impl Entity for GodJson {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn with_id(&mut self, id: i32) -> Self {
        self.id = Some(id);
        self.clone()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CharacterJson {
    pub id: Option<i32>,
    pub name: String,
    pub description: String,
    pub health: i32,
    pub attack: i32,
    pub cost: u8,
    pub upgrade: Option<CharacterUpgrade>,
    pub abilities: Vec<Ability>,
}

impl ToTokens for CharacterJson {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let abilities = self
            .abilities
            .iter()
            .map(|a| ability_as_tokens(&a))
            .collect::<Vec<_>>();

        let id = self.id.unwrap_or(0);
        let name = &self.name;
        let description = &self.description;
        let health = self.health;
        let attack = self.attack;
        let cost = self.cost;
        let upgrade = if let Some(upgrade) = &self.upgrade {
            upgrade_as_tokens(upgrade)
        } else {
            quote! { None }
        };
        tokens.append_all(quote! {
            Character {
                id: #id,
                name: #name.to_string(),
                description: #description.to_string(),
                health: #health,
                attack: #attack,
                cost: #cost,
                upgrade: #upgrade,
                abilities: vec![#(#abilities),*],
            }
        });
    }
}

fn upgrade_as_tokens(upgrade: &CharacterUpgrade) -> TokenStream {
    let name = &upgrade.name;
    let attack = upgrade.attack;
    let health = upgrade.health;
    let abilities = upgrade
        .abilities
        .iter()
        .map(|a| ability_as_tokens(&a))
        .collect::<Vec<_>>();
    quote! {
        Some(CharacterUpgrade {
            name: #name.to_string(),
            attack: #attack,
            health: #health,
            abilities: vec![#(#abilities),*],
        })
    }
}

fn ability_as_tokens(ability: &Ability) -> TokenStream {
    let name = &ability.name;
    let description = &ability.description;
    let trigger = format_ident!("{}", format!("{:?}", ability.trigger));
    let target = format_ident!("{}", format!("{:?}", ability.target));
    let effect = ability_effect_as_tokens(&ability.effect);
    quote! {
        Ability {
            name: #name.to_string(),
            description: #description.to_string(),
            trigger: AbilityTrigger::#trigger,
            target: AbilityTarget::#target,
            effect: #effect,
        }
    }
}

// impl ToString for CharacterJson {
//     fn to_string(&self, idx: usize) -> String {
//         format!(
//             "Character {{ id: {}, name: Borrowed(\"{}\"), description: Borrowed(\"{}\"), health: {}, attack: {}, cost: {}, upgrade: {}, abilities: {}}}",
//             idx,
//             self.name,
//             self.description,
//             self.health,
//             self.attack,
//             self.cost,
//             if let Some(upgrade) = &self.upgrade { format!("Some(CharacterUpgrade {{ name: Borrowed(\"{}\"), attack: {}, health: {}, abilities: {} }})", upgrade.name, upgrade.attack, upgrade.health, format_abilities(&upgrade.abilities)) } else { "None".to_string() },
//             format_abilities(&self.abilities)
//         )
//     }
// }

impl Entity for CharacterJson {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn with_id(&mut self, id: i32) -> Self {
        self.id = Some(id);
        self.clone()
    }
}

// fn format_abilities(abilities: &Vec<Ability>) -> String {
//     format!(
//         "vec![{}]",
//         abilities
//             .iter()
//             .map(|ability| format_ability(ability))
//             .collect::<Vec<String>>()
//             .join(", ")
//     )
// }

// fn format_ability(ability: &Ability) -> String {
//     format!("Ability {{ name: Borrowed(\"{}\"), description: Borrowed(\"{}\"), trigger: AbilityTrigger::{:?}, effect: {}, target: AbilityTarget::{:?} }}", ability.name, ability.description, ability.trigger, format_ability_effect(&ability.effect), ability.target)
// }

fn ability_effect_as_tokens(effect: &AbilityEffect) -> TokenStream {
    match effect {
        AbilityEffect::Summon(_) => todo!(),
        AbilityEffect::Transform(_) => todo!(),
        AbilityEffect::Buff(attack, health, is_permanent) => {
            let attack = ability_value_as_tokens(attack);
            let health = ability_value_as_tokens(health);
            quote! {
                AbilityEffect::Buff(
                    #attack,
                    #health,
                    #is_permanent
                )
            }
        }
        AbilityEffect::Set(_, _) => todo!(),
        AbilityEffect::Damage(_) => todo!(),
        AbilityEffect::Slience(_) => todo!(),
        AbilityEffect::Stun(_) => todo!(),
        AbilityEffect::Stealth => todo!(),
        AbilityEffect::Taunt(_) => todo!(),
        AbilityEffect::Ranged => todo!(),
        AbilityEffect::Flying => quote! {
            AbilityEffect::Flying
        },
        AbilityEffect::FirstStrike => quote! {
            AbilityEffect::FirstStrike
        },
    }
}

fn ability_value_as_tokens(value: &AbilityValue) -> TokenStream {
    match value {
        AbilityValue::Plain(value) => quote! {
            AbilityValue::Plain(#value)
        },
        AbilityValue::PercentHealth(value) => quote! {
            AbilityValue::PercentHealth(#value)
        },
        AbilityValue::PercentAttack(value) => quote! {
            AbilityValue::PercentAttack(#value)
        },
        AbilityValue::PercentMaxHealth(value) => quote! {
            AbilityValue::PercentMaxHealth(#value)
        },
        AbilityValue::PercentMaxAttack(value) => quote! {
            AbilityValue::PercentMaxAttack(#value)
        },
        AbilityValue::PercentTargetHealth(value) => quote! {
            AbilityValue::PercentTargetHealth(#value)
        },
        AbilityValue::PercentTargetAttack(value) => quote! {
            AbilityValue::PercentTargetAttack(#value)
        },
        AbilityValue::PercentTargetMaxHealth(value) => quote! {
            AbilityValue::PercentTargetMaxHealth(#value)
        },
        AbilityValue::PercentTargetMaxAttack(value) => quote! {
            AbilityValue::PercentTargetMaxAttack(#value)
        },
    }
}
