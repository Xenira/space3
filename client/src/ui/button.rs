use bevy::{hierarchy::ChildBuilder, math::Size, prelude::*, ui::Style};

use crate::{
    util::colors::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    Id,
};

#[derive(Debug)]
pub struct ButtonClickEvent(pub String);

pub fn generate_button(
    text: &str,
    id: &str,
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) -> Entity {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
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
                ..Default::default()
            });
        })
        .insert(Id(id.to_string()))
        .id()
}

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &Id, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mouse_buttons: Res<Input<MouseButton>>,
    mut ev_button_click: EventWriter<ButtonClickEvent>,
) {
    for (interaction, button_id, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if mouse_buttons.just_released(MouseButton::Left) {
                    ev_button_click.send(ButtonClickEvent(button_id.0.clone()));
                }
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
