use bevy::prelude::*;
use protocol::protocol::GameUserInfo;

pub(crate) struct GameUserInfoPlugin;

impl Plugin for GameUserInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_user_info_added.run_if(resource_added::<GameUserRes>()))
            .add_system(on_user_info_update.run_if(resource_exists_and_changed::<GameUserRes>()))
            .add_system(on_user_info_removed.run_if(resource_removed::<GameUserRes>()));
    }
}

#[derive(Resource, Debug)]
pub struct GameUserRes(pub GameUserInfo);

#[derive(Component, Debug)]
pub struct UserHealth;

#[derive(Component, Debug)]
pub struct UserMoney;

#[derive(Component, Debug)]
pub struct UserExperience;

fn on_user_info_added(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_user_info: Res<GameUserRes>,
) {
    info!("Game user info added");
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("Health: {}", game_user_info.0.health),
                TextStyle {
                    font: asset_server.load("fonts/monogram-extended.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..Default::default()
        },
        UserHealth,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("Exp: {}", game_user_info.0.experience),
                TextStyle {
                    font: asset_server.load("fonts/monogram-extended.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, -20.0, 10.0)),
            ..Default::default()
        },
        UserExperience,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("$: {}", game_user_info.0.money),
                TextStyle {
                    font: asset_server.load("fonts/monogram-extended.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, -40.0, 10.0)),
            ..Default::default()
        },
        UserMoney,
    ));
}

fn on_user_info_update(
    game_user_info: Res<GameUserRes>,
    mut q_set: ParamSet<(
        Query<&mut Text, With<UserHealth>>,
        Query<&mut Text, With<UserExperience>>,
        Query<&mut Text, With<UserMoney>>,
    )>,
) {
    info!("Game user info updated: {:?}", game_user_info);
    for mut text in q_set.p0().iter_mut() {
        text.sections[0].value = format!("Health: {}", game_user_info.0.health);
    }
    for mut text in q_set.p1().iter_mut() {
        text.sections[0].value = format!("Exp: {}", game_user_info.0.experience);
    }
    for mut text in q_set.p2().iter_mut() {
        text.sections[0].value = format!("$: {}", game_user_info.0.money);
    }
}

fn on_user_info_removed(
    mut commands: Commands,
    q_info: Query<Entity, Or<(With<UserHealth>, With<UserExperience>, With<UserMoney>)>>,
) {
    info!("Game user info removed");
    for entity in q_info.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
