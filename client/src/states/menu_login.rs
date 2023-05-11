use bevy::{
    app::AppExit,
    prelude::{shape::Quad, *},
    utils::tracing::field::debug,
};
use bevy_egui::{egui, EguiContexts};
use protocol::protocol::{Credentials, Protocol, UserData};
use protocol::protocol_types::heros;
use surf::http::Method;

use crate::{
    cleanup_system,
    components::{
        animation::{AnimationIndices, AnimationTimer, TransformAnimation},
        hover::{BoundingBox, ClickEvent, Hoverable, Hovered},
    },
    networking::{
        networking_events::NetworkingEvent, networking_ressource::NetworkingRessource,
        polling::PollingStatus,
    },
    prefabs::animation,
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MenuLogin;
pub(crate) struct MenuLoginPlugin;

impl Plugin for MenuLoginPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoginCredentials>()
            .add_system(logout.in_schedule(OnEnter(STATE)))
            .add_systems((ui_login, on_login, god_hover, god_click).in_set(OnUpdate(STATE)))
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

    let god_frame = asset_server.load("textures/ui/god_frame2.png");
    let god_frame_atlas =
        TextureAtlas::from_grid(god_frame, Vec2::new(64.0, 64.0), 18, 1, None, None);
    let god_frame_atlas_handle = texture_atlases.add(god_frame_atlas);

    let mut frame_animation = animation::simple(0, 0);
    animation::add_hover_state(&mut frame_animation, 0, 17);

    let god_fallback = asset_server.load("textures/ui/god_fallback.png");

    commands
        .spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-64.0 * 4.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_translation(Vec3::new(0.0, 33.0 * 4.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    God(protocol::gods::GODS[0].clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                            .with_translation(Vec3::new(33.0 * 4.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    God(protocol::gods::GODS[1].clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            90.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                            .with_translation(Vec3::new(-33.0 * 4.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation.clone(),
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    God(protocol::gods::GODS[2].clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            -90.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: god_frame_atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        transform: Transform::from_scale(Vec3::splat(4.0))
                            .with_rotation(Quat::from_rotation_z(180.0f32.to_radians()))
                            .with_translation(Vec3::new(0.0, -33.0 * 4.0, 1.0)),
                        ..Default::default()
                    },
                    Hoverable("hover".to_string(), "leave".to_string()),
                    BoundingBox(
                        Vec3::new(48.0, 48.0, 0.0),
                        Quat::from_rotation_z(45.0f32.to_radians()),
                    ),
                    frame_animation,
                    AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
                    God(protocol::gods::GODS[3].clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: god_fallback.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            180.0f32.to_radians(),
                        )),
                        ..Default::default()
                    });
                });
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::End,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        gap: Size::height(Val::Px(24.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Select your deity",
                            TextStyle {
                                font: asset_server.load("fonts/monogram-extended.ttf"),
                                font_size: 50.0,
                                color: Color::WHITE,
                            },
                        ),
                        GodTitle,
                    ));
                    parent.spawn((
                        TextBundle::from_sections([TextSection::from_style(TextStyle {
                            font: asset_server.load("fonts/monogram-extended.ttf"),
                            font_size: 28.0,
                            color: Color::WHITE,
                        })]),
                        GodDescription,
                    ));
                    parent.spawn((
                        TextBundle::from_sections([TextSection::from_style(TextStyle {
                            font: asset_server.load("fonts/monogram-extended.ttf"),
                            font_size: 32.0,
                            color: Color::WHITE,
                        })]),
                        GodPantheon,
                    ));
                });
        });

    debug!("Logout end")
}

#[derive(Component, Debug)]
pub struct GodTitle;

#[derive(Component, Debug)]
pub struct GodDescription;

#[derive(Component, Debug)]
pub struct GodPantheon;

#[derive(Component, Debug)]
pub struct God(heros::God);

fn god_hover(
    q_hovered: Query<&God, Added<Hovered>>,
    mut q_title: Query<&mut Text, With<GodTitle>>,
    mut q_desc: Query<&mut Text, (With<GodDescription>, Without<GodTitle>)>,
    mut q_pantheon: Query<
        &mut Text,
        (
            With<GodPantheon>,
            Without<GodTitle>,
            Without<GodDescription>,
        ),
    >,
) {
    if let Some(god) = q_hovered.iter().next() {
        q_title.get_single_mut().unwrap().sections[0].value = god.0.name.clone();
        q_desc.get_single_mut().unwrap().sections[0].value =
            break_text(god.0.description.clone(), 36, true);
        q_pantheon.get_single_mut().unwrap().sections[0].value = god.0.pantheon.clone();
    }
}

fn god_click(
    mut commands: Commands,
    mut ev_clicked: EventReader<ClickEvent>,
    q_god: Query<(Entity, &Transform), With<God>>,
) {
    for ev in ev_clicked.iter() {
        q_god.get(ev.0).ok().map(|(god, transform)| {
            commands.entity(god).insert(TransformAnimation {
                target: transform.with_translation(Vec3::ZERO),
                speed: 1.0,
            });
        });
    }
}

fn break_text(text: String, width: usize, center: bool) -> String {
    text.split(" ")
        .fold(vec!["".to_string()], |mut acc: Vec<String>, word| {
            let current_line = acc.last_mut().unwrap();
            if current_line.len() + word.len() > width {
                acc.push(word.to_string());
            } else {
                if current_line.len() > 0 {
                    current_line.push_str(" ");
                }
                current_line.push_str(word);
            }
            acc
        })
        .iter()
        .map(|line| {
            format!(
                "{}{}",
                " ".repeat(if center { (width - line.len()) / 2 } else { 0 }),
                line
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
