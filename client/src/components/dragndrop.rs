use super::{
    cursor::Cursor,
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
                (on_drag, on_drop)
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
pub struct Dragged {
    pub original_transform: Transform,
    pub original_parent: Option<Entity>,
}

#[derive(Debug)]
pub struct DragEvent(pub Entity);

#[derive(Debug)]
pub struct DropEvent {
    pub target: Entity,
    pub entity: Entity,
}

fn on_drag(
    mut commands: Commands,
    mut ev_clicked: EventReader<ClickEvent>,
    mut q_hovered: Query<
        (Entity, &mut Transform, &GlobalTransform, Option<&Parent>),
        (With<Clickable>, With<Dragable>),
    >,
    mut ev_draged: EventWriter<DragEvent>,
    q_cursor: Query<(Entity, &GlobalTransform), With<Cursor>>,
) {
    for ev in ev_clicked.iter() {
        if let Ok((entity, mut transform, global_transform, parent)) = q_hovered.get_mut(ev.0) {
            debug!("Dragged: {:?}", entity);
            let (cursor, cursor_transform) = q_cursor.single();

            commands.entity(entity).insert(Dragged {
                original_transform: transform.clone(),
                original_parent: parent.map(|p| p.get()),
            });
            commands.entity(entity).remove_parent();
            commands.entity(cursor).add_child(entity);
            transform.translation = Vec3::ZERO;

            // Add inverse of cursor transform to the entity transform + 5% scale keeping z
            let global_scale = global_transform.compute_transform().scale;
            transform.scale = global_scale.truncate().extend(0.0)
                * (1.0 / cursor_transform.compute_transform().scale)
                * 1.05
                + Vec3::new(0.0, 0.0, global_scale.z);

            ev_draged.send(DragEvent(entity));
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
                debug!("Droped: {:?}", entity);
                commands.entity(entity).remove::<Dragged>();
                if let Some(parent) = dragged.original_parent {
                    commands.entity(entity).set_parent(parent);
                }
                commands
                    .entity(entity)
                    .insert(dragged.original_transform.clone());

                if let Ok((drop_target, _)) = q_drop_target.get_single() {
                    debug!("Droped on: {:?}", drop_target);
                    ev_droped.send(DropEvent {
                        target: drop_target,
                        entity,
                    });
                }
            }
        }
    }
}
