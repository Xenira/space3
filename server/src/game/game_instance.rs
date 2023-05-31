use chrono::{DateTime, Utc};
use protocol::protocol::{BattleResponse, GameOpponentInfo, Protocol, Turn};
use uuid::Uuid;

use crate::{
    model::polling::ActivePolls,
    service::{combat_service, game_service, simple_bot_service},
};

use super::{
    game_instance_player::GameInstancePlayer, COMBAT_DURATION_MULTIPLIER, DEFAULT_COMBAT_DURATION,
};

#[derive(Debug)]
pub struct GameInstance {
    pub game_id: Uuid,
    pub players: [GameInstancePlayer; 8],
    pub turn: Turn,
}

impl GameInstance {
    pub fn new(players: [GameInstancePlayer; 8]) -> Self {
        Self {
            game_id: Uuid::new_v4(),
            players,
            turn: Turn::default(),
        }
    }

    pub fn get_user(&self, user_id: i32) -> Option<&GameInstancePlayer> {
        self.players
            .iter()
            .find(|player| player.user_id == Some(user_id))
    }

    pub fn get_game_user(&self, id: Uuid) -> Option<&GameInstancePlayer> {
        self.players.iter().find(|player| player.id == id)
    }

    pub fn get_user_mut(&mut self, user_id: i32) -> Option<&mut GameInstancePlayer> {
        self.players
            .iter_mut()
            .find(|player| player.user_id == Some(user_id))
    }

    pub fn get_game_user_mut(&mut self, id: Uuid) -> Option<&mut GameInstancePlayer> {
        self.players.iter_mut().find(|player| player.id == id)
    }

    pub fn has_user(&self, user_id: i32) -> bool {
        self.players
            .iter()
            .any(|player| player.user_id == Some(user_id))
    }

    // TODO: Move back to service
    pub async fn next_turn(&mut self) -> bool {
        let (turn_time, ended) = match self.turn {
            Turn::Combat(turn, _) => self.start_shop(turn + 1).await,
            Turn::Shop(_, _) => (self.start_combat().await, false),
        };

        if ended {
            return true;
        }

        self.turn.next(turn_time);
        false
    }

    async fn start_shop(&mut self, _turn: u16) -> (DateTime<Utc>, bool) {
        game_service::update_player_placements(self).await;

        if self.is_game_over() {
            return (Utc::now(), true);
        }

        let turn: u16 = self.turn.into();
        for player in self.players.iter_mut() {
            player.experience += 1;
            player.generate_shop();
            player.money = (turn + 2).min(16);
        }

        simple_bot_service::perform_bot_turns(self).await;

        (
            Utc::now() + chrono::Duration::seconds(90.min(30 + (turn as i64 / 2 - 1) * 5)),
            false,
        )
    }

    async fn start_combat(&mut self) -> DateTime<Utc> {
        let pairings = combat_service::get_pairing(
            self.turn.into(),
            self.players.as_ref().iter().collect::<Vec<_>>(),
        );

        let mut combat_duration = DEFAULT_COMBAT_DURATION;

        for pairing in pairings {
            if let (Some(player_a), Some(player_b)) = self
                .players
                .iter_mut()
                .filter(|player| player.id == pairing.0 || player.id == pairing.1)
                .enumerate()
                .fold((None, None), |curr, (i, p)| {
                    if i == 0 {
                        (Some(p), curr.1)
                    } else {
                        (curr.0, Some(p))
                    }
                })
            {
                combat_duration = ((Self::execute_combat((player_a, player_b)).await as f64
                    * COMBAT_DURATION_MULTIPLIER) as i64)
                    .max(combat_duration);
            }
        }

        Utc::now() + chrono::Duration::seconds(combat_duration)
    }

    // TODO: Move back to service
    async fn execute_combat(
        mut pairing: (&mut GameInstancePlayer, &mut GameInstancePlayer),
    ) -> usize {
        let user_b_id = pairing.1.user_id;
        let player_a_op_info = GameOpponentInfo {
            name: pairing.0.display_name.clone(),
            health: pairing.0.health,
            experience: pairing.0.experience,
            character_id: pairing.0.god.clone().unwrap().id,
            is_next_opponent: false,
        };
        let player_b_op_info = GameOpponentInfo {
            name: pairing.1.display_name.clone(),
            health: pairing.1.health,
            experience: pairing.1.experience,
            character_id: pairing.1.god.clone().unwrap().id,
            is_next_opponent: false,
        };

        let (actions, start_own, start_opponent) =
            combat_service::calculate_combat(&mut pairing).await;
        let combat_result = BattleResponse {
            actions,
            start_own,
            start_opponent,
            opponent: player_b_op_info,
        };

        let mut swapped_result = combat_result.swap_players();
        swapped_result.opponent = player_a_op_info;

        let action_len = combat_result.actions.len();

        if pairing.0.placement.is_none() && pairing.0.user_id.is_some() {
            ActivePolls::notify(
                pairing.0.user_id.unwrap(),
                Protocol::GameBattleResponse(combat_result),
            )
            .await;
        }

        if pairing.1.placement.is_none() && user_b_id.is_some() {
            ActivePolls::notify(
                user_b_id.unwrap(),
                Protocol::GameBattleResponse(swapped_result),
            )
            .await;
        }

        action_len
    }

    pub fn is_game_over(&self) -> bool {
        self.players
            .iter()
            .filter(|player| player.health > 0)
            .count()
            <= 1
    }
}
