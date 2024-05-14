use bevy::prelude::*;
use bevy_ecss::StyleSheet;

use crate::{
    states::startup::startup::{UiAssets, UiRoot},
    Cleanup, UiCamera,
};

use super::button::{ButtonComponent, ButtonProperties};

const MIN_WIDTH: f32 = 200.0;
const MIN_HEIGHT: f32 = 100.0;

const TITLE_SIZE: f32 = 32.0;
const BODY_SIZE: f32 = 22.0;

const BORDER_COLOR_ACTIVE: Color = Color::rgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

pub(crate) struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_spawn);
    }
}

#[derive(Component, Default, Debug)]
pub struct Dialog {
    pub title: Option<String>,
    pub text: Option<String>,
    pub buttons: ButtonGroup,
    pub content: Option<Entity>,
    pub backdrop: bool,
}

#[derive(Debug, Default)]
pub struct ButtonGroup {
    pub buttons: Vec<ButtonProperties>,
    pub direction: FlexDirection,
}

fn on_spawn(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    q_added: Query<&Dialog, Added<Dialog>>,
    q_ui_root: Query<Entity, With<UiRoot>>,
) {
    for dialog in q_added.iter() {
        spawn_dialog(
            &mut commands,
            ui_assets.dialog_border.clone(),
            dialog,
            q_ui_root.single(),
        );
    }
}

fn spawn_dialog(
    commands: &mut Commands,
    background: Handle<Image>,
    dialog: &Dialog,
    ui_root: Entity,
) {
    let slicer = TextureSlicer {
        border: BorderRect::square(22.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };

    let dialog_parent = commands
        .spawn((
            NodeBundle {
                style: Style {
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            Cleanup,
        ))
        .id();

    let dialog_body = commands
        .spawn((
            ImageBundle {
                style: Style {
                    min_height: Val::Px(MIN_HEIGHT),
                    min_width: Val::Px(MIN_WIDTH),
                    max_width: Val::Px(500.0),
                    max_height: Val::Px(500.0),
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(22.0)),
                    ..default()
                },
                image: UiImage::new(background),
                ..default()
            },
            ImageScaleMode::Sliced(slicer.clone()),
        ))
        .id();

    let title = dialog.title.as_ref().map(|title| {
        commands
            .spawn(TextBundle::from_section(
                title,
                TextStyle {
                    color: Color::BLACK,
                    font_size: TITLE_SIZE,
                    ..default()
                },
            ))
            .id()
    });

    let text = dialog.text.as_ref().map(|text| {
        commands
            .spawn(TextBundle::from_section(
                text,
                TextStyle {
                    color: Color::DARK_GRAY,
                    font_size: BODY_SIZE,
                    ..default()
                },
            ))
            .id()
    });

    let buttons = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: dialog.buttons.direction,
                align_self: AlignSelf::Stretch,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for button in &dialog.buttons.buttons {
                parent.spawn((NodeBundle::default(), ButtonComponent(button.clone())));
            }
        })
        .id();

    {
        let mut dialog_body = commands.entity(dialog_body);
        if let Some(title) = title {
            dialog_body.add_child(title);
        }
        if let Some(text) = text {
            dialog_body.add_child(text);
        }
        if let Some(content) = dialog.content {
            dialog_body.add_child(content);
        }
        dialog_body.add_child(buttons);
    }

    commands.entity(dialog_parent).add_child(dialog_body);
    commands.entity(ui_root).add_child(dialog_parent);
}
