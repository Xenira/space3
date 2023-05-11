use crate::{
    cleanup_system,
    components::animation::{AnimationIndices, AnimationTimer},
    AppState, Cleanup,
};
use bevy::prelude::*;
use protocol::protocol_types::heros::God;

const STATE: AppState = AppState::GameCommanderSelection;

pub(crate) struct GameCommanderSelectionPlugin;

impl Plugin for GameCommanderSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource)]
pub(crate) struct GameCommanderSelection(pub Vec<God>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // let god_frame = asset_server.load("textures/ui/god_frame.png");
    // let god_frame_atlas =
    //     TextureAtlas::from_grid(god_frame, Vec2::new(64.0, 64.0), 5, 5, None, None);
    // let god_frame_atlas_handle = texture_atlases.add(god_frame_atlas);
    // let god_frame_animation_indices = AnimationIndices { first: 1, last: 21 };

    // commands.spawn((
    //     SpriteSheetBundle {
    //         texture_atlas: god_frame_atlas_handle,
    //         sprite: TextureAtlasSprite::new(god_frame_animation_indices.first),
    //         transform: Transform::from_scale(Vec3::splat(0.0)),
    //         ..Default::default()
    //     },
    //     god_frame_animation_indices,
    //     AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    // ));
}
