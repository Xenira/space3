use super::{
    hover::{BoundingBox, ClickEvent, Clickable, Hovered},
    ChangeDetectionSystemSet,
};
use crate::MainCamera;
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    transform,
};

pub(crate) struct DragNDropPlugin;

impl Plugin for DragNDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DragEvent>()
            .add_event::<DropEvent>()
            .add_systems(
                (on_drag, drag, on_drop)
                    .chain()
                    .in_set(ChangeDetectionSystemSet::MouseDetection),
            );
    }
}

#[derive(Debug, Component, Clone)]
pub struct Dragable;

#[derive(Debug, Component, Clone)]
pub struct DropTagret;

#[derive(Debug, Component, Clone)]
pub struct Dragged(pub Transform);

#[derive(Debug)]
pub struct DragEvent(pub Entity);

#[derive(Debug)]
pub struct DropEvent(pub Entity);

fn on_drag(
    mut commands: Commands,
    mut ev_clicked: EventReader<ClickEvent>,
    q_hovered: Query<(Entity, &Transform), (With<Clickable>, With<Dragable>)>,
    mut ev_draged: EventWriter<DragEvent>,
) {
    for ev in ev_clicked.iter() {
        if let Ok((entity, transform)) = q_hovered.get(ev.0) {
            commands.entity(entity).insert(Dragged(transform.clone()));
            ev_draged.send(DragEvent(entity));
        }
    }
}

fn drag(
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ev_cursor_move: EventReader<CursorMoved>,
    mut q_draged: Query<(&mut Transform, &GlobalTransform), With<Dragged>>,
) {
    let (camera, camera_transform) = q_camera.single();
    for cursor_event in ev_cursor_move.iter() {
        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, cursor_event.position)
            .map(|r| r.origin)
        {
            for (mut transform, global_transform) in q_draged.iter_mut() {
                transform.translation =
                    world_position - global_transform.compute_transform().translation;
            }
        }
    }
}

fn on_drop(
    mut commands: Commands,
    mut ev_cursor_click: EventReader<MouseButtonInput>,
    q_draged: Query<(Entity, &Dragged)>,
    q_drop_target: Query<(Entity, &GlobalTransform), (With<DropTagret>, With<Hovered>)>,
    mut ev_droped: EventWriter<DropEvent>,
) {
    for ev in ev_cursor_click.iter() {
        if ev.button == MouseButton::Left && ev.state == ButtonState::Released {
            for (entity, dragged) in q_draged.iter() {
                commands.entity(entity).remove::<Dragged>();
                if let Ok((drop_target, _)) = q_drop_target.get_single() {
                    ev_droped.send(DropEvent(drop_target));
                } else {
                    commands.entity(entity).insert(dragged.0.clone());
                }
            }
        }
    }
}
