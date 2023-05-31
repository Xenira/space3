use crate::components::{
    animation::{
        Animation, AnimationBundle, AnimationDirection, AnimationIndices, AnimationRepeatType,
        AnimationState, AnimationTimer,
    },
    timer::TimerTextComponent,
};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct TimerBundle {
    #[bundle]
    pub animation_bundle: AnimationBundle,
}

impl TimerBundle {
    pub fn new(asset_server: &AssetServer, texture_atlases: &mut Assets<TextureAtlas>) -> Self {
        let frame = asset_server.load("textures/ui/timer_frame.png");
        let frame_atlas = TextureAtlas::from_grid(frame, Vec2::new(128.0, 64.0), 25, 1, None, None);
        let frame_atlas_handle = texture_atlases.add(frame_atlas);

        let init_state = AnimationState::new("init", AnimationIndices::new(0, 24))
            .with_repeat_type(AnimationRepeatType::Once);
        let frame_animation = Animation::default()
            .with_state(init_state)
            .with_state(
                AnimationState::new("close", AnimationIndices::new(0, 24))
                    .with_repeat_type(AnimationRepeatType::Once)
                    .with_direction(AnimationDirection::Backward),
            )
            .with_current_state("init");

        let animation_bundle = AnimationBundle {
            animation: frame_animation,
            animation_timer: AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: frame_atlas_handle.clone(),
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_scale(Vec3::splat(1.0)),
                ..Default::default()
            },
        };
        Self { animation_bundle }
    }
}

#[derive(Bundle)]
pub struct TimerTextBundle {
    pub timer_text: TimerTextComponent,

    #[bundle]
    pub text: Text2dBundle,
}

impl TimerTextBundle {
    pub fn new(asset_server: &AssetServer, transform: Transform) -> Self {
        Self {
            timer_text: TimerTextComponent,
            text: Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/monogram-extended.ttf"),
                        font_size: 50.0,
                        color: Color::WHITE,
                    },
                ),
                transform,
                ..Default::default()
            },
        }
    }
}
