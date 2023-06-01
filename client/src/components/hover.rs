use super::{animation::Animation, cursor::Cursor, ChangeDetectionSystemSet};
use crate::components::animation::AnimationTransition;
use bevy::{input::mouse::MouseButtonInput, prelude::*};
use std::ops::Mul;

pub(crate) struct HoverPlugin;

impl Plugin for HoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClickEvent>()
            .add_systems(
                (check_hover, on_hover_added, on_hover_removed)
                    .chain()
                    .in_set(ChangeDetectionSystemSet::MouseDetection),
            )
            .add_system(on_click.after(check_hover));
    }
}

#[derive(Debug, Component, Clone)]
pub struct Hovered;

#[derive(Debug, Component, Clone)]
pub struct BoundingBox(pub Vec3, pub Quat);

impl From<Vec2> for BoundingBox {
    fn from(size: Vec2) -> Self {
        Self(size.extend(0.0), Quat::IDENTITY)
    }
}

#[derive(Debug, Component, Clone)]
pub struct Clickable;

pub struct ClickEvent(pub Entity);

impl BoundingBox {
    pub fn is_point_inside(&self, point: Vec2, transform: &Transform) -> bool {
        let relative_position = point.extend(0.0) - transform.translation;
        let rotated_relative_position = transform.rotation.mul_vec3(relative_position);
        let rotated_point = self.1.mul_vec3(rotated_relative_position).mul(2.0).abs();
        // debug!("rotated_point: {:?}", rotated_point);
        rotated_point.x >= 0.0
            && rotated_point.x <= self.0.x * transform.scale.x + 1.0
            && rotated_point.y >= 0.0
            && rotated_point.y <= self.0.y * transform.scale.y + 1.0
    }
}

#[derive(Component, Debug, Clone)]
pub struct Hoverable(pub String, pub String);

impl Default for Hoverable {
    fn default() -> Self {
        Self("hover".to_string(), "leave".to_string())
    }
}

fn check_hover(
    mut commands: Commands,
    mut ev_cursor_move: EventReader<CursorMoved>,
    mut q_bounding_boxes: Query<
        (Entity, &BoundingBox, &GlobalTransform, Option<&Hovered>),
        With<Hoverable>,
    >,
    q_cursor: Query<&mut Transform, With<Cursor>>,
) {
    if ev_cursor_move.iter().next().is_some() {
        if let Ok(world_position) = q_cursor
            .get_single()
            .map(|cursor| cursor.translation.truncate())
        {
            for (entity, bounding_box, transform, hovered) in q_bounding_boxes.iter_mut() {
                if bounding_box.is_point_inside(world_position, &transform.compute_transform()) {
                    if hovered.is_none() {
                        trace!("Hovering over entity: {:?}", entity);
                        commands.entity(entity).insert(Hovered);
                    }
                } else if hovered.is_some() {
                    trace!("No longer hovering over entity: {:?}", entity);
                    commands.entity(entity).remove::<Hovered>();
                }
            }
        }
    }
}

fn on_hover_added(
    mut commands: Commands,
    query: Query<(Entity, &Hoverable, &Animation), Added<Hovered>>,
) {
    for (entity, hoverable, animation) in &query {
        if let Some(transition) = animation.get_transition(&hoverable.0) {
            commands.entity(entity).insert(transition.clone());
        } else {
            warn!(
                "No transition found for hoverable {:?} in {:?}",
                hoverable, animation
            );
        }
    }
}

fn on_hover_removed(
    mut commands: Commands,
    mut removed: RemovedComponents<Hovered>,
    query: Query<(Entity, &Hoverable, &Animation, Option<&AnimationTransition>), Without<Hovered>>,
) {
    for removed in removed.iter() {
        if let Ok((entity, hoverable, animation, existing_transition)) = query.get(removed) {
            if let Some(transition) = animation.get_transition(&hoverable.1) {
                debug!(
                    "Removing hoverable {:?} from entity {:?} with transition {:?} -> {:?}",
                    hoverable, entity, existing_transition, transition
                );
                commands.entity(entity).insert(transition.clone());
            } else {
                warn!(
                    "No transition found for hoverable {:?} in {:?}",
                    hoverable, animation
                );
            }
        }
    }
}

fn on_click(
    mut ev_cursor_click: EventReader<MouseButtonInput>,
    q_hovered: Query<Entity, (With<Hovered>, With<Clickable>)>,
    mut ev_click: EventWriter<ClickEvent>,
) {
    for cursor_event in ev_cursor_click.iter() {
        if cursor_event.button == MouseButton::Left && cursor_event.state.is_pressed() {
            for entity in q_hovered.iter() {
                debug!("Clicked on entity: {:?}", entity);
                ev_click.send(ClickEvent(entity));
            }
        }
    }
}

#[test]
fn did_handle_hover_add() {
    // TODO: Test this
    // let mut app = App::new();
    // app.add_plugin(HoverPlugin);
    // app.add_startup_system(setup.system());
    // app.add_system(check_hover.system());
    // app.add_system(on_hover_added.system());
    // app.add_system(on_hover_removed.system());
    // app.update();

    // let entity = app.world.spawn().insert_bundle(SpriteBundle {
    // 	transform: Transform::from_xyz(0.0, 0.0, 0.0),
    // 	..Default::default()
    // }).insert(Hoverable("idle".to_string(), "hover".to_string()));

    // fn setup(mut commands: Commands) {
    // 	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // 	commands.spawn_bundle(SpriteBundle {
    // 		transform: Transform::from_xyz(0.0, 0.0, 0.0),
    // 		..Default::default()
    // 	}).insert(Hoverable("idle".to_string(), "hover".to_string()))
    // 	.insert(Animation::new("idle".to_string(), vec![
    // 		AnimationTransition::new("hover".to_string(), "hover".to_string(), 0.0, 0.0),
    // 	]));
    // }

    // assert!(app.world.get::<Hovered>(entity).is_some());
}

#[test]
fn did_calculate_bounding_box_colision() {
    let bounding_box = BoundingBox(Vec3::new(1.0, 1.0, 0.0), Quat::IDENTITY);
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);
    assert!(bounding_box.is_point_inside(Vec2::new(0.0, 0.0), &transform));
    assert!(bounding_box.is_point_inside(Vec2::new(0.5, 0.5), &transform));
    assert!(bounding_box.is_point_inside(Vec2::new(1.0, 1.0), &transform));
    assert!(!bounding_box.is_point_inside(Vec2::new(1.1, 1.1), &transform));
    assert!(!bounding_box.is_point_inside(Vec2::new(1.0, 1.1), &transform));
    assert!(!bounding_box.is_point_inside(Vec2::new(1.1, 1.0), &transform));
    assert!(bounding_box.is_point_inside(Vec2::new(0.0, 1.0), &transform));
    assert!(bounding_box.is_point_inside(Vec2::new(1.0, 0.0), &transform));
    assert!(bounding_box.is_point_inside(Vec2::new(0.0, 0.0), &transform));
}
