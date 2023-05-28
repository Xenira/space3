use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub id: i32,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub health: i32,
    pub attack: i32,
    pub cost: u8,
    pub upgrade: Option<CharacterUpgrade>,
    pub abilities: Vec<Ability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterUpgrade {
    pub name: Cow<'static, str>,
    pub attack: i32,
    pub health: i32,
    pub abilities: Vec<Ability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ability {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub trigger: AbilityTrigger,
    pub effect: AbilityEffect,
    pub target: AbilityTarget,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityTrigger {
    OnAttack,
    OnDeath,
    OnSurvive,
    Passive,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityEffect {
    // Summon a character with the given id
    Summon(i32),
    // Transform into a character with the given id
    Transform(i32),
    // Buff/Debuff the given character
    Buff(AbilityValue, AbilityValue, bool),
    // Set the given character stats
    Set(AbilityValue, AbilityValue),
    // Damage the given character
    Damage(AbilityValue),
    // Disable the characters ability for X triggers
    Slience(u8),
    // Stun the character for X turns
    Stun(u8),
    // Character can't be targeted until it attacks
    Stealth,
    // Characters target this character if able for X turns
    Taunt(u8),
    // Character doesn't take damage from attacking
    Ranged,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityTarget {
    SelfTarget,
    EnemyTarget,
    AllyTarget,
    AllEnemyTarget,
    AllAllyTarget,
    AllTarget,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityValue {
    Plain(i32),
    PercentHealth(i32),
    PercentAttack(i32),
    PercentMaxHealth(i32),
    PercentMaxAttack(i32),
    PercentTargetHealth(i32),
    PercentTargetAttack(i32),
    PercentTargetMaxHealth(i32),
    PercentTargetMaxAttack(i32),
}
