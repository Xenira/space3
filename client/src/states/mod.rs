pub(crate) mod game_commander_selection;
pub(crate) mod game_search;
pub(crate) mod game_shop;
pub(crate) mod menu_login;
pub(crate) mod menu_main;

pub(crate) mod game_states {
    use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

    use super::{game_commander_selection, game_search, game_shop, menu_login, menu_main};

    pub(crate) struct GameStatesPluginGroup;

    impl PluginGroup for GameStatesPluginGroup {
        fn build(&mut self, group: &mut PluginGroupBuilder) {
            group
                .add(menu_login::MenuLoginPlugin)
                .add(menu_main::MenuMainPlugin)
                .add(game_search::GameSearchPlugin)
                .add(game_commander_selection::GameCommanderSelectionPlugin)
                .add(game_shop::GameShopPlugin);
        }
    }
}
