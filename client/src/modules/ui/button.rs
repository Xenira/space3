use bevy::prelude::*;

use crate::{states::startup::startup::UiAssets, UiCamera, UI_LAYER};

const PRIMAY_FONT_SIZE: f32 = 18.0;

pub(crate) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_spawn, button_system))
            .add_event::<ButtonClickEvent>();
    }
}

#[derive(Component, Debug)]
pub struct ButtonComponent(pub ButtonProperties);

#[derive(Component, Debug)]
pub struct ButtonRoleComponent(pub ButtonRole);

#[derive(Clone, Debug)]
pub struct ButtonProperties {
    pub text: String,
    pub role: ButtonRole,
    pub id: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub enum ButtonRole {
    #[default]
    Primary,
    Secondary,
    Cancel,
}

#[derive(Clone, Default, Hash, Eq, PartialEq, Debug)]
pub enum ButtonState {
    #[default]
    None,
    Hovered,
    Pressed,
}

impl Into<ButtonState> for &Interaction {
    fn into(self) -> ButtonState {
        match self {
            Interaction::None => ButtonState::None,
            Interaction::Hovered => ButtonState::Hovered,
            Interaction::Pressed => ButtonState::Pressed,
        }
    }
}

#[derive(Event, Debug)]
pub struct ButtonClickEvent {
    pub button: ButtonProperties,
}

fn on_spawn(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    q_added: Query<(&ButtonComponent, &Parent), Added<ButtonComponent>>,
) {
    for (button, parent) in q_added.iter() {
        spawn_button(&mut commands, &ui_assets, &button, parent.get());
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &Parent, &mut UiImage, &ButtonRoleComponent),
        (Changed<Interaction>, With<Button>),
    >,
    q_button: Query<&ButtonComponent>,
    q_children: Query<&Children>,
    // mut q_image: Query<&mut UiImage>,
    // q_role: Query<&ButtonRoleComponent>,
    res_assets: Res<UiAssets>,
    mut ev_button_click: EventWriter<ButtonClickEvent>,
) {
    for (interaction, parent, mut image, role) in &mut interaction_query {
        // let mut text = text_query.get_mut(children[0]).unwrap();
        // let mut image = q_image.get_mut(children[0]).unwrap();
        // let role = q_role.get(children[0]).unwrap();
        match role.0 {
            ButtonRole::Primary => {
                image.texture = res_assets
                    .primary_button
                    .get(&interaction.into())
                    .unwrap()
                    .clone();
            }
            ButtonRole::Secondary => {
                image.texture = res_assets
                    .secondary_button
                    .get(&interaction.into())
                    .unwrap()
                    .clone();
            }
            ButtonRole::Cancel => {
                image.texture = res_assets
                    .cancel_button
                    .get(&interaction.into())
                    .unwrap()
                    .clone();
            }
        }

        if *interaction == Interaction::Pressed {
            q_children
                .get(parent.get())
                .iter()
                .flat_map(|children| children.iter().filter_map(|c| q_button.get(*c).ok()))
                .for_each(|button| {
                    ev_button_click.send(ButtonClickEvent {
                        button: button.0.clone(),
                    });
                });
        }
    }
}

fn spawn_button(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    button: &ButtonComponent,
    entity: Entity,
) {
    let slicer = TextureSlicer {
        border: BorderRect::square(22.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    image: UiImage::new(ui_assets.dialog_border.clone()),
                    // background_color: Color::rgba(1.0, 1.0, 1.0, 0.05).into(),
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
                ButtonRoleComponent(button.0.role.clone()),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    button.0.text.clone(),
                    TextStyle {
                        color: Color::BLACK,
                        font_size: PRIMAY_FONT_SIZE,
                        ..default()
                    },
                ));
            });
    });
}
