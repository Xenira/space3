use bevy::prelude::*;

use crate::{states::startup::UiAssets, AppState, MainCamera};

pub(crate) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnExit(AppState::Startup)))
            .add_system(update.in_base_set(CoreSet::First));
    }
}

#[derive(Component)]
pub struct Cursor;

fn setup(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(5.0, -10.0, 900.0)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    texture: ui_assets.cursor.clone(),
                    transform: Transform::from_scale(Vec3::splat(0.1)),
                    ..Default::default()
                },
                Cursor,
            ));
        });
}

fn update(
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ev_cursor_move: EventReader<CursorMoved>,
    mut q_cursor: Query<&mut Transform, With<Cursor>>,
) {
    let (camera, camera_transform) = q_camera.single();
    if let Some(cursor_event) = ev_cursor_move.iter().last() {
        if let Some(world_position) =
            camera.viewport_to_world_2d(camera_transform, cursor_event.position)
        {
            if let Ok(mut cursor) = q_cursor.get_single_mut() {
                cursor.translation = world_position.extend(0.0);
            }
        }
    }
}
