pub(crate) mod game_commander_selection;
pub(crate) mod menu_lobby_join;
// pub(crate) mod game_search;
pub(crate) mod game_combat;
pub(crate) mod game_result;
pub(crate) mod game_shop;
pub(crate) mod menu_lobby;
pub(crate) mod menu_login;
pub(crate) mod menu_main;
pub(crate) mod menu_settings;
pub(crate) mod set_display_name;
pub(crate) mod startup;

pub(crate) mod game_states {
    use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

    use super::*;

    pub(crate) struct GameStatesPluginGroup;

    impl PluginGroup for GameStatesPluginGroup {
        fn build(self) -> PluginGroupBuilder {
            PluginGroupBuilder::start::<Self>()
                .add(menu_login::MenuLoginPlugin)
                .add(set_display_name::SetDisplayNamePlugin)
                .add(menu_main::MenuMainPlugin)
                .add(menu_settings::MenuSettingsPlugin)
                .add(menu_lobby_join::DialogLobbyJoinPlugin)
                .add(menu_lobby::LobbyPlugin)
                // .add(game_search::GameSearchPlugin)
                .add(game_commander_selection::GameCommanderSelectionPlugin)
                .add(game_shop::GameShopPlugin)
                .add(game_combat::GameCombatPlugin)
                .add(game_result::GameResultPlugin)
                .add(startup::startup::StartupPlugin)
                .add(startup::loading::LoadingPlugin)
        }
    }
}
