use std::collections::HashMap;

use crate::{
    components::background::Background, modules::ui::button::ButtonState, AppState,
    StateChangeEvent,
};
use bevy::prelude::*;

const STATE: AppState = AppState::Startup;
pub(crate) struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .init_resource::<BackgroundAssets>()
            .add_systems(
                OnEnter(STATE),
                (load_ui_assets, load_background_assets).before(setup),
            )
            .add_systems(OnEnter(STATE), setup);
    }
}

#[derive(Resource, Default)]
pub struct UiAssets {
    pub font: Handle<Font>,
    pub cursor: Handle<Image>,
    pub dialog_border: Handle<Image>,
    pub primary_button: HashMap<ButtonState, Handle<Image>>,
    pub secondary_button: HashMap<ButtonState, Handle<Image>>,
    pub cancel_button: HashMap<ButtonState, Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct BackgroundAssets {
    pub(crate) background: Handle<Image>,
}

fn load_ui_assets(asset_server: Res<AssetServer>, mut ui_assets: ResMut<UiAssets>) {
    ui_assets.font = asset_server.load("fonts/monogram-extended.ttf");
    ui_assets.cursor = asset_server.load("textures/ui/cursor.png");
    ui_assets.dialog_border = asset_server.load("textures/ui/windows/dialog.png");
    ui_assets.primary_button = load_button_assets("btn_primary", &asset_server);
    ui_assets.secondary_button = load_button_assets("btn_secondary", &asset_server);
    ui_assets.cancel_button = load_button_assets("btn_cancel", &asset_server);
}

fn load_button_assets(
    name: &str,
    asset_server: &Res<AssetServer>,
) -> HashMap<ButtonState, Handle<Image>> {
    let mut button = HashMap::new();
    button.insert(
        ButtonState::None,
        asset_server.load(format!("textures/ui/windows/{}.png", name)),
    );
    button.insert(
        ButtonState::Hovered,
        asset_server.load(format!("textures/ui/windows/{}_hover.png", name)),
    );
    button.insert(
        ButtonState::Pressed,
        asset_server.load(format!("textures/ui/windows/{}_click.png", name)),
    );
    button
}

fn load_background_assets(
    asset_server: Res<AssetServer>,
    mut background_assets: ResMut<BackgroundAssets>,
) {
    background_assets.background = asset_server.load("textures/background/bg.png");
}

fn setup(
    mut ev_state_change: EventWriter<StateChangeEvent>,
    background_assets: ResMut<BackgroundAssets>,
    mut background_resource: ResMut<Background>,
) {
    ev_state_change.send(StateChangeEvent(AppState::Loading));
    background_resource.0 = background_assets.background.clone();
}
