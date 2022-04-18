use bevy::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct FocusNextEvent {
    pub form: Option<String>,
    pub idx: usize,
    pub sender: Option<Entity>,
}

pub struct FocusEvent {
    pub entity: Entity,
    pub old_entity: Option<Entity>,
}

#[derive(Component)]
pub struct Focus;

#[derive(Component, Default)]
pub struct FocusTarget(FocusNextEvent);

#[derive(Bundle, Default)]
pub struct FocusBundle {
    target: FocusTarget,
    interaction: Interaction,
}

impl FocusBundle {
    pub fn new(idx: usize) -> Self {
        Self {
            target: FocusTarget(FocusNextEvent {
                idx,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FocusEvent>()
            .add_event::<FocusNextEvent>()
            .add_system(on_focus_next)
            .add_system(focus_system)
            .add_system(keyboard_input);
    }
}

fn on_focus_next(
    mut commands: Commands,
    mut ev_focus_next: EventReader<FocusNextEvent>,
    mut next_query: Query<(&FocusTarget, Entity), Without<Focus>>,
    mut ev_focus: EventWriter<FocusEvent>,
    current_query: Query<Entity, With<Focus>>,
) {
    for ev in ev_focus_next.iter() {
        debug!("Focus Event {:?}", ev);
        if let Some((_, entity)) = next_query
            .iter()
            .filter(|(t, _)| t.0.idx >= ev.idx)
            .min_by_key(|(t, _)| (t.0.idx as i32 - ev.idx as i32).abs())
            .or(next_query.iter().min_by_key(|(t, _)| t.0.idx))
        {
            set_focus(
                entity,
                ev.sender,
                &current_query,
                &mut commands,
                &mut ev_focus,
            );
        }
    }
}

fn set_focus(
    target: Entity,
    sender: Option<Entity>,
    current_query: &Query<Entity, With<Focus>>,
    commands: &mut Commands,
    ev_focus: &mut EventWriter<FocusEvent>,
) {
    debug!("New focus {:?} -> {:?}", sender, target);
    for e in current_query.iter() {
        commands.entity(e).remove::<Focus>();
    }
    commands.entity(target).insert(Focus {});
    ev_focus.send(FocusEvent {
        entity: target,
        old_entity: sender,
    });
}

pub fn focus_system(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, Entity), (Changed<Interaction>, Without<Focus>)>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut ev_focus: EventWriter<FocusEvent>,
    current_query: Query<Entity, With<Focus>>,
) {
    for (interaction, entity) in interaction_query.iter() {
        debug!("Interacted {:?}", interaction);
        if let Interaction::Hovered = *interaction {
            if mouse_buttons.just_released(MouseButton::Left) {
                let old_entity = current_query.get_single();
                set_focus(
                    entity,
                    old_entity.ok(),
                    &current_query,
                    &mut commands,
                    &mut ev_focus,
                );
            }
        }
    }
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut ev_focus_next: EventWriter<FocusNextEvent>,
    current_query: Query<(&FocusTarget, Entity), With<Focus>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        if let Ok((focus, current)) = current_query.get_single() {
            ev_focus_next.send(FocusNextEvent {
                sender: Some(current),
                ..focus.0.clone()
            });
        } else {
            ev_focus_next.send(FocusNextEvent::default());
        }
    }
}
