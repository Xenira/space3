use super::{
    anchors::{AnchorType, Anchors},
    ChangeDetectionSystemSet,
};
use bevy::{prelude::*, window::PrimaryWindow};

pub(crate) struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetTooltipEvent>()
            .add_startup_system(setup.in_base_set(CoreSet::Last))
            .add_systems(
                (on_cursor_move, on_set_tooltip_event)
                    .in_set(ChangeDetectionSystemSet::TooltipRender),
            );
    }
}

#[derive(Component)]
pub struct Tooltip(pub AnchorType, Option<Entity>);

pub struct SetTooltipEvent(pub Entity, pub Option<Entity>);

fn setup(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 50.0),
            ..Default::default()
        },
        Tooltip(AnchorType::MIDDLE, None),
    ));
}

fn on_cursor_move(
    mut commands: Commands,
    mut ev_cursor_move: EventReader<CursorMoved>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_tooltip: Query<(&mut Tooltip, Entity)>,
    res_anchor: Res<Anchors>,
) {
    if let Some(x) = ev_cursor_move.iter().last().map(|e| e.position.x) {
        let width = q_window.get_single().unwrap().width();
        let (mut anchor, tooltip) = q_tooltip.get_single_mut().unwrap();

        if x < width / 2.0 && anchor.0 & AnchorType::RIGHT != AnchorType::RIGHT {
            trace!("Setting anchor to MIDDLE_RIGHT");
            if let Some(anchor_entity) = res_anchor.get(AnchorType::MIDDLE_RIGHT) {
                anchor.0 = AnchorType::MIDDLE_RIGHT;
                commands.entity(tooltip).set_parent(anchor_entity);
            }
        } else if x > width / 2.0 && anchor.0 & AnchorType::LEFT != AnchorType::LEFT {
            trace!("Setting anchor to MIDDLE_LEFT");
            if let Some(anchor_entity) = res_anchor.get(AnchorType::MIDDLE_LEFT) {
                anchor.0 = AnchorType::MIDDLE_LEFT;
                commands.entity(tooltip).set_parent(anchor_entity);
            }
        }
    }
}

fn on_set_tooltip_event(
    mut commands: Commands,
    mut ev: EventReader<SetTooltipEvent>,
    mut q_tooltip: Query<(&mut Tooltip, Entity)>,
) {
    for SetTooltipEvent(ref_entity, new_tooltip_entity) in ev.iter() {
        let (mut tooltip, tooltip_entity) = q_tooltip.get_single_mut().unwrap();

        if let Some(new_tooltip_entity) = new_tooltip_entity {
            tooltip.1 = Some(*ref_entity);
            commands.entity(tooltip_entity).despawn_descendants();
            commands
                .entity(*new_tooltip_entity)
                .set_parent(tooltip_entity);
        } else if tooltip.1 == Some(*ref_entity) {
            debug!("Despawning tooltip");
            commands.entity(tooltip_entity).despawn_descendants();
        }
    }
}
