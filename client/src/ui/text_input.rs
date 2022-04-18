use std::cmp;

use bevy::{
    hierarchy::ChildBuilder,
    input::keyboard::KeyboardInput,
    math::Size,
    prelude::*,
    ui::{FocusPolicy, Style},
};

use crate::{
    ui::form::{FormElementChangedEvent, FormMapping, FormValue},
    util::colors::NORMAL_BUTTON,
};

use super::{
    focus::{Focus, FocusBundle, FocusEvent, FocusNextEvent, FocusTarget},
    form::{FormElement, FormElements, FormId},
};

pub struct TextEditSubmitEvent {
    pub value: String,
}

#[derive(Component, Debug)]
pub struct TextEdit {
    value: String,
    placeholder: Option<String>,
    mask: Option<String>,
}

impl TextEdit {}

#[derive(Component)]
pub struct InputFocus(usize);

pub fn add_to_system_set(set: SystemSet) -> SystemSet {
    set.with_system(update_text)
        .with_system(focus_system)
        .with_system(text_input)
        .with_system(add_cursor)
        .with_system(remove_cursor)
}

pub(crate) struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TextEditSubmitEvent>()
            .add_system_to_stage(CoreStage::PostUpdate, remove_cursor);
    }
}

#[derive(Bundle)]
struct TextInputBundle {
    id: FormId,
    text_input: TextEdit,

    #[bundle]
    target: FocusBundle,

    #[bundle]
    node: NodeBundle,
}

pub fn generate_input(
    id: &str,
    idx: usize,
    text: &str,
    placeholder: Option<String>,
    mask: Option<String>,
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) -> Entity {
    parent
        .spawn_bundle(TextInputBundle {
            node: NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(350.0), Val::Px(65.0)),
                    // center button
                    margin: Rect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                color: NORMAL_BUTTON.into(),
                focus_policy: FocusPolicy::Block,
                ..Default::default()
            },
            id: FormId(id.to_string()),
            text_input: TextEdit {
                value: text.to_string(),
                placeholder: placeholder.clone(),
                mask,
            },
            target: FocusBundle::new(idx),
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    text,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                focus_policy: FocusPolicy::Pass,

                ..Default::default()
            });
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                visibility: Visibility { is_visible: false },
                focus_policy: FocusPolicy::Pass,

                ..Default::default()
            });
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                focus_policy: FocusPolicy::Pass,

                ..Default::default()
            });
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    placeholder.unwrap_or("".to_string()).as_str(),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgba(0.9, 0.9, 0.9, 0.5),
                    },
                    Default::default(),
                ),
                focus_policy: FocusPolicy::Pass,

                ..Default::default()
            });
        })
        .id()
}

/// Print the stats of friendly players when they change
fn add_cursor(
    mut commands: Commands,
    old_focus: Query<Entity, With<InputFocus>>,
    new_focus: Query<(&TextEdit, Entity, &Children), Added<InputFocus>>,
    mut visibility_query: Query<(&mut Visibility, &mut Text)>,
) {
    if let Some((text, entity, children)) = new_focus.iter().last() {
        for entity in old_focus.iter().filter(|e| *e != entity) {
            commands.entity(entity).remove::<InputFocus>();
        }

        if let Some(seperator) = children.get(1) {
            if let Ok((mut vis, mut text)) = visibility_query.get_mut(*seperator) {
                vis.is_visible = true;
                text.sections[0].value = "_".to_string();
            }
        }
        if let Some(placeholder) = children.get(3) {
            if let Ok((_, mut t)) = visibility_query.get_mut(*placeholder) {
                set_placeholder(text, &mut t, true);
            }
        }
    }
}

/// Print the stats of friendly players when they change
pub fn remove_cursor(
    removed: RemovedComponents<InputFocus>,
    children_query: Query<(&TextEdit, &Children)>,
    mut visibility_query: Query<(&mut Visibility, &mut Text), With<Text>>,
) {
    for entity in removed.iter() {
        debug!("Entity {:?} removed", entity);
        for (text_edit, children) in children_query.get(entity).iter() {
            if let Some(child) = children.iter().nth(1) {
                if let Ok((mut vis, mut text)) = visibility_query.get_mut(*child) {
                    vis.is_visible = false;
                    text.sections[0].value = "".to_string();
                }
            }
            if let Some(placeholder) = children.iter().nth(3) {
                if let Ok((_, mut t)) = visibility_query.get_mut(*placeholder) {
                    set_placeholder(text_edit, &mut t, false);
                }
            }
        }
    }
}

/// Print the stats of friendly players when they change
fn update_text(
    query: Query<
        (Option<&InputFocus>, &TextEdit, &Children),
        Or<(Changed<TextEdit>, Changed<InputFocus>)>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (focus, text, children) in query.iter() {
        let orig_text = if let Some(mask) = &text.mask {
            mask.repeat(text.value.len())
        } else {
            text.value.clone()
        };
        let texts = if let Some(f) = focus {
            orig_text.split_at(f.0)
        } else {
            (orig_text.as_str(), "")
        };
        debug!("Text updated {} -> {:?}", orig_text, texts);

        if let Some(t1) = children.get(0) {
            if let Ok(mut text) = text_query.get_mut(*t1) {
                text.sections[0].value = texts.0.to_string();
            } else {
                warn!("Failed to set text to {:?} for t1 {:?}", texts.0, t1);
            }
        }
        if let Some(t2) = children.get(2) {
            match text_query.get_mut(*t2) {
                Ok(mut text) => {
                    text.sections[0].value = texts.1.to_string();
                }
                Err(e) => {
                    warn!("Failed to set text to {:?} for t2 {:?}: {}", texts.0, t2, e);
                }
            }
        }
        if let Some(placeholder) = children.get(3) {
            if let Ok(mut t) = text_query.get_mut(*placeholder) {
                set_placeholder(text, &mut t, focus.is_some());
            }
        }
    }
}

fn set_placeholder(text_edit: &TextEdit, text: &mut Text, focused: bool) {
    let placeholder_text = if focused || text_edit.value.len() > 0 {
        "".to_string()
    } else if let Some(placeholder) = &text_edit.placeholder {
        placeholder.clone()
    } else {
        "".to_string()
    };
    text.sections[0].value = placeholder_text;
}

pub fn focus_system(
    mut commands: Commands,
    mut ev_focus: EventReader<FocusEvent>,
    text_edit_query: Query<&TextEdit>,
) {
    for ev in ev_focus.iter() {
        debug!("Focus event recieved");
        if let Some(old_entity) = ev.old_entity {
            debug!("Unfocusing {:?}", old_entity);
            commands.entity(old_entity).remove::<InputFocus>();
        }

        if let Ok(text) = text_edit_query.get(ev.entity) {
            debug!("Setting focus to {:?}", text);

            commands
                .entity(ev.entity)
                .insert(InputFocus(text.value.len()));
        }
    }
}

fn text_input(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    mut editable_query: Query<(&mut TextEdit, &mut InputFocus, &FormId, Entity), With<InputFocus>>,
    mut ev_submit: EventWriter<TextEditSubmitEvent>,
    mut ev_changed: EventWriter<FormElementChangedEvent>,
) {
    let mut ctrl_key_pressed = true;

    for keycode in keys.get_just_pressed() {
        match keycode {
            KeyCode::Back => {
                if let Some((mut text, mut focus, id, entity)) = editable_query.iter_mut().nth(0) {
                    if focus.0 > 0 {
                        text.value.remove(focus.0 - 1);
                        send_text_event(id, entity, text.value.as_str(), &mut ev_changed);
                        focus.0 = cmp::max(0, cmp::min(text.value.len(), focus.0 - 1));
                    }
                }
            }
            KeyCode::Delete => {
                if let Some((mut text, mut focus, id, entity)) = editable_query.iter_mut().nth(0) {
                    if text.value.len() > focus.0 {
                        text.value.remove(focus.0);
                        send_text_event(id, entity, text.value.as_str(), &mut ev_changed);
                        focus.0 = cmp::max(0, cmp::min(text.value.len(), focus.0));
                    }
                }
            }
            KeyCode::Left => {
                if let Some((text, mut focus, _, _)) = editable_query.iter_mut().nth(0) {
                    if focus.0 > 0 {
                        focus.0 = cmp::max(0, cmp::min(text.value.len(), focus.0 - 1));
                    }
                }
            }
            KeyCode::Right => {
                if let Some((text, mut focus, _, _)) = editable_query.iter_mut().nth(0) {
                    focus.0 = cmp::max(0, cmp::min(text.value.len(), focus.0 + 1));
                }
            }
            KeyCode::Return => {
                if let Some((text, _, _, _)) = editable_query.iter().nth(0) {
                    ev_submit.send(TextEditSubmitEvent {
                        value: text.value.clone(),
                    })
                }
            }
            KeyCode::Tab => {}
            _ => ctrl_key_pressed = false,
        }
    }
    if !ctrl_key_pressed {
        for ev in char_evr.iter().filter(|c| !c.char.is_control()) {
            if let Ok((mut text, mut focus, id, entity)) = editable_query.get_single_mut() {
                text.value.insert(focus.0, ev.char);
                send_text_event(id, entity, text.value.as_str(), &mut ev_changed);
                focus.0 += 1;
            }
            println!("Got char: '{}'", ev.char);
        }
    }
}

fn send_text_event(
    id: &FormId,
    entity: Entity,
    value: &str,
    ev_changed: &mut EventWriter<FormElementChangedEvent>,
) {
    let mut form_mapping = FormElements::new();
    form_mapping.insert(
        id.0.clone(),
        FormElement {
            value: Some(FormValue::String(value.to_string())),
            entity: Some(entity),
        },
    );
    ev_changed.send(FormElementChangedEvent(form_mapping));
}
