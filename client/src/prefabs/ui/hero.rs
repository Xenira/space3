use bevy::prelude::*;

use crate::components::{
    animation::AnimationBundle,
    hover::{BoundingBox, Hoverable},
};

#[derive(Bundle)]
pub struct HeroSelectionChoiceBundle {
    pub hoverable: Hoverable,
    pub bounding_box: BoundingBox,

    #[bundle]
    pub animation_bundle: AnimationBundle,
}

// impl HeroSelectionChoiceBundle {
//     pub fn new(frame: Handle<TextureAtlas>) -> Self {
//         let mut frame_animation = animation::simple(0, 0);
//         animation::add_hover_state(&mut frame_animation, 0, 17);
//         Self {
//             hoverable: Hoverable::default(),
//             bounding_box: BoundingBox(
//                 Vec3::new(48.0, 48.0, 0.0),
//                 Quat::from_rotation_z(45.0f32.to_radians()),
//             ),
//             animation_bundle: AnimationBundle {
//                 animation: frame_animation,
//                 animation_timer: AnimationTimer(Timer::from_seconds(0.1, true)),
//                 sprite_sheet: SpriteSheetBundle {
//                     texture_atlas: TextureAtlas::from_grid(
//                         asset_server.load("textures/ui/god_frame2.png"),
//                         Vec2::new(64.0, 64.0),
//                         18,
//                         1,
//                         None,
//                         None,
//                     ),
//                     transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
//                     sprite: TextureAtlasSprite::new(0),
//                     ..Default::default()
//                 },
//             },
//         }
//     }
// }

pub fn hero_selection() {}

pub fn hero_selection_frame(index: u8) {}
