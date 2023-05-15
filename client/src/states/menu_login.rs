use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use protocol::protocol::{Credentials, Protocol, UserData};

use surf::http::Method;

use crate::{
    cleanup_system,
    components::{
        animation::AnimationTimer,
        dragndrop::{Dragable, DropTagret},
        hover::{BoundingBox, Clickable, Hoverable},
    },
    networking::{
        networking_events::NetworkingEvent, networking_ressource::NetworkingRessource,
        polling::PollingStatus,
    },
    prefabs::ui::timer::{TimerBundle, TimerTextBundle},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MenuLogin;
pub(crate) struct MenuLoginPlugin;

impl Plugin for MenuLoginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoginCredentials>()
            .add_system(logout.in_schedule(OnEnter(STATE)))
            .add_systems((ui_login, on_login).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource, Default)]
struct LoginCredentials(Credentials);

#[derive(Resource)]
struct User(UserData);

fn logout(
    mut commands: Commands,
    mut ev_polling_status: EventWriter<PollingStatus>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    debug!("Logout start");
    commands.remove_resource::<User>();
    ev_polling_status.send(PollingStatus::Stop);

    // commands.spawn(TimerBundle::new(
    //     Timer::from_seconds(30.0, TimerMode::Once),
    //     &asset_server,
    //     texture_atlases.as_mut(),
    // ));
    commands.spawn(TimerTextBundle::new(&asset_server));

    let shop_frame = asset_server.load("textures/ui/user_frame.png");
    let shop_frame_atlas =
        TextureAtlas::from_grid(shop_frame, Vec2::new(64.0, 64.0), 2, 1, None, None);
    let shop_frame_atlas_handle = texture_atlases.add(shop_frame_atlas);

    let pedestal = asset_server.load("textures/ui/character_base.png");
    let pedestal_atlas = TextureAtlas::from_grid(pedestal, Vec2::new(64.0, 64.0), 2, 1, None, None);
    let pedestal_atlas_handle = texture_atlases.add(pedestal_atlas);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 0.0, 0.0)),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteSheetBundle {
                    texture_atlas: pedestal_atlas_handle.clone(),
                    sprite: TextureAtlasSprite::new(0),
                    transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(
                        68.0 * 3.0 as f32,
                        0.0,
                        1.0,
                    )),
                    ..Default::default()
                },
                Hoverable("hover".to_string(), "leave".to_string()),
                BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                DropTagret,
            ));

            parent.spawn((
                SpriteSheetBundle {
                    texture_atlas: shop_frame_atlas_handle.clone(),
                    sprite: TextureAtlasSprite::new(0),
                    transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(
                        68.0 * 1.0 as f32,
                        200.0,
                        1.0,
                    )),
                    ..Default::default()
                },
                Hoverable("hover".to_string(), "leave".to_string()),
                BoundingBox(Vec3::new(64.0, 64.0, 0.0), Quat::from_rotation_z(0.0)),
                Dragable,
                Clickable,
            ));
        });

    debug!("Logout end")
}

fn ui_login(
    mut contexts: EguiContexts,
    mut network: ResMut<NetworkingRessource>,
    mut credentials: ResMut<LoginCredentials>,
    mut ev_exit: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Login");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut credentials.0.username);
        });
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut credentials.0.password).password(true));
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Login").clicked() {
                network.request_data(Method::Post, "users", &credentials.0);
            }
            if ui.button("Register").clicked() {
                network.request_data(Method::Put, "users", &credentials.0);
            }
        });
        if ui.button("Exit").clicked() {
            ev_exit.send(AppExit);
        }
    });
}

fn on_login(
    mut commands: Commands,
    mut network: ResMut<NetworkingRessource>,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_polling_status: EventWriter<PollingStatus>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        if let Protocol::LoginResponse(login) = &ev.0 {
            network.client = network
                .client
                .config()
                .clone()
                .add_header("x-api-key", login.key.clone())
                .unwrap()
                .try_into()
                .unwrap();
            network.polling_client = network
                .polling_client
                .config()
                .clone()
                .add_header("x-api-key", login.key.clone())
                .unwrap()
                .try_into()
                .unwrap();
            commands.insert_resource(User(login.user.clone()));
            debug!("Logged in as {}", login.user.username);

            ev_polling_status.send(PollingStatus::Start);
            ev_state_change.send(StateChangeEvent(AppState::MenuMain));
        }
    }
}
