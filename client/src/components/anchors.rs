use crate::{AppState, MainCamera};
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bitflags::bitflags;

pub(crate) struct AnchorsPlugin;

impl Plugin for AnchorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Anchors>()
            .add_system(
                setup
                    .in_base_set(CoreSet::First)
                    .in_schedule(OnEnter(AppState::Startup)),
            )
            .add_system(on_resize);
    }
}

#[derive(Resource, Default, Debug)]
pub struct Anchors {
    anchors: HashMap<AnchorType, Entity>,
}

impl Anchors {
    pub fn get(&self, anchor_type: AnchorType) -> Option<Entity> {
        self.anchors.get(&anchor_type).copied()
    }

    pub fn set(&mut self, anchor_type: AnchorType, entity: Entity) {
        self.anchors.insert(anchor_type, entity);
    }
}

#[derive(Component, Debug)]
pub struct Anchor {
    pub anchor_type: AnchorType,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AnchorType: u8 {
        const TOP = 0b00000001;
        const BOTTOM = 0b00000010;
        const LEFT = 0b00000100;
        const RIGHT = 0b00001000;
        const MIDDLE = 0b00010000;
        const TOP_LEFT = Self::TOP.bits() | Self::LEFT.bits();
        const TOP_RIGHT = Self::TOP.bits() | Self::RIGHT.bits();
        const BOTTOM_LEFT = Self::BOTTOM.bits() | Self::LEFT.bits();
        const BOTTOM_RIGHT = Self::BOTTOM.bits() | Self::RIGHT.bits();
        const MIDDLE_LEFT = Self::MIDDLE.bits() | Self::LEFT.bits();
        const MIDDLE_RIGHT = Self::MIDDLE.bits() | Self::RIGHT.bits();
    }
}

fn setup(
    mut commands: Commands,
    mut anchors: ResMut<Anchors>,
    mut q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let camera = q_camera.single_mut();

    let window = q_window.single_mut();
    let width = window.width();
    let height = window.height();

    anchors.set(
        AnchorType::TOP,
        spawn_anchor(&mut commands, (width, height), camera, AnchorType::TOP),
    );
    anchors.set(
        AnchorType::BOTTOM,
        spawn_anchor(&mut commands, (width, height), camera, AnchorType::BOTTOM),
    );
    anchors.set(
        AnchorType::LEFT,
        spawn_anchor(&mut commands, (width, height), camera, AnchorType::LEFT),
    );
    anchors.set(
        AnchorType::RIGHT,
        spawn_anchor(&mut commands, (width, height), camera, AnchorType::RIGHT),
    );
    anchors.set(
        AnchorType::TOP_LEFT,
        spawn_anchor(&mut commands, (width, height), camera, AnchorType::TOP_LEFT),
    );
    anchors.set(
        AnchorType::TOP_RIGHT,
        spawn_anchor(
            &mut commands,
            (width, height),
            camera,
            AnchorType::TOP_RIGHT,
        ),
    );
    anchors.set(
        AnchorType::BOTTOM_LEFT,
        spawn_anchor(
            &mut commands,
            (width, height),
            camera,
            AnchorType::BOTTOM_LEFT,
        ),
    );
    anchors.set(
        AnchorType::BOTTOM_RIGHT,
        spawn_anchor(
            &mut commands,
            (width, height),
            camera,
            AnchorType::BOTTOM_RIGHT,
        ),
    );

    anchors.set(
        AnchorType::MIDDLE_LEFT,
        spawn_anchor(
            &mut commands,
            (width, height),
            camera,
            AnchorType::MIDDLE_LEFT,
        ),
    );

    anchors.set(
        AnchorType::MIDDLE_RIGHT,
        spawn_anchor(
            &mut commands,
            (width, height),
            camera,
            AnchorType::MIDDLE_RIGHT,
        ),
    );
}

fn on_resize(
    mut q_anchor: Query<(&Anchor, &mut Transform)>,
    mut q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let window = q_window.single_mut();
    let width = window.width();
    let height = window.height();
    let (camera, global_transform) = q_camera.single_mut();
    for (anchor, mut transform) in q_anchor.iter_mut() {
        if let Some(position) = camera.viewport_to_world_2d(
            global_transform,
            get_sceen_space_position((width, height), anchor.anchor_type),
        ) {
            transform.translation = position.extend(0.0);
        }
    }
}

fn spawn_anchor(
    commands: &mut Commands,
    (width, height): (f32, f32),
    (camera, camera_transform): (&Camera, &GlobalTransform),
    anchor_type: AnchorType,
) -> Entity {
    let position = camera
        .viewport_to_world_2d(
            camera_transform,
            get_sceen_space_position((width, height), anchor_type),
        )
        .unwrap();

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(position.extend(0.0)),
                ..Default::default()
            },
            Anchor { anchor_type },
        ))
        .id()
}

fn get_sceen_space_position((width, height): (f32, f32), anchor_type: AnchorType) -> Vec2 {
    Vec2::new(
        if anchor_type & AnchorType::LEFT == AnchorType::LEFT {
            if anchor_type & AnchorType::MIDDLE == AnchorType::MIDDLE {
                width / 4.0
            } else {
                0.0
            }
        } else if anchor_type & AnchorType::RIGHT == AnchorType::RIGHT {
            if anchor_type & AnchorType::MIDDLE == AnchorType::MIDDLE {
                width / 4.0 * 3.0
            } else {
                width
            }
        } else {
            width / 2.0
        },
        if anchor_type & AnchorType::BOTTOM == AnchorType::BOTTOM {
            0.0
        } else if anchor_type & AnchorType::TOP == AnchorType::TOP {
            height
        } else {
            height / 2.0
        },
    )
}
