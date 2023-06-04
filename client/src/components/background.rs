use bevy::{prelude::*, window::PrimaryWindow};

const BACKGROUND_WIDTH: f32 = 1280.0;
const BACKGROUND_HEIGHT: f32 = 720.0;

pub(crate) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Background>()
            .add_startup_system(setup)
            .add_systems((on_background_change, on_window_resize));
    }
}

#[derive(Resource, Default)]
pub struct Background(pub Handle<Image>);

#[derive(Component)]
pub struct BackgroundComponent;

fn setup(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        BackgroundComponent,
    ));
}

fn on_background_change(
    mut commands: Commands,
    q_background: Query<Entity, With<BackgroundComponent>>,
    texture: Res<Background>,
) {
    if texture.is_changed() {
        commands
            .entity(q_background.get_single().unwrap())
            .insert(SpriteBundle {
                texture: texture.0.clone(),
                ..Default::default()
            });
    }
}

fn on_window_resize(
    mut q_background: Query<&mut Transform, With<BackgroundComponent>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut last_scale: Local<f32>,
) {
    if let Ok(window) = q_window.get_single() {
        let width = window.width() / BACKGROUND_WIDTH;
        let height = window.height() / BACKGROUND_HEIGHT;
        let scale = width.max(height);
        if scale != *last_scale {
            *last_scale = scale;
            q_background.get_single_mut().unwrap().scale = Vec3::splat(scale);
        }
    }
}
