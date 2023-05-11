use chrono::NaiveDateTime;
use diesel::Queryable;

#[derive(Queryable)]
pub struct GameUserCharacters {
	pub game_user_id: u32,
	pub character_id: u32,
	pub position: u8,
	pub upgraded: bool,
	pub attack_bonus: i32,
	pub defense_bonus: i32,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}