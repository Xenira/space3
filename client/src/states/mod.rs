pub(crate) mod dialog_lobby_join;
pub(crate) mod game_commander_selection;
// pub(crate) mod game_search;
pub(crate) mod game_shop;
pub(crate) mod lobby;
pub(crate) mod menu_login;
pub(crate) mod menu_main;

pub(crate) mod game_states {
    use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

    use super::*;

    pub(crate) struct GameStatesPluginGroup;

    impl PluginGroup for GameStatesPluginGroup {
        fn build(self) -> PluginGroupBuilder {
            PluginGroupBuilder::start::<Self>()
                .add(menu_login::MenuLoginPlugin)
                .add(menu_main::MenuMainPlugin)
                .add(dialog_lobby_join::DialogLobbyJoinPlugin)
                .add(lobby::LobbyPlugin)
                // .add(game_search::GameSearchPlugin)
                .add(game_commander_selection::GameCommanderSelectionPlugin)
                .add(game_shop::GameShopPlugin)
        }
    }
}
