use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};

use crate::{cleanup_system, AppState, Cleanup, StateChangeEvent};

const STATE: AppState = AppState::MenuMain;
pub(crate) struct MenuMainPlugin;

impl Plugin for MenuMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_main_menu.in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

fn ui_main_menu(
    mut contexts: EguiContexts,
    mut ev_state_change: EventWriter<StateChangeEvent>,
    mut ev_exit: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Main Menu");
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Play").clicked() {
                ev_state_change.send(StateChangeEvent(AppState::DialogLobbyJoin));
            }
            if ui.button("Exit").clicked() {
                ev_exit.send(AppExit);
            }
        });
    });
}
