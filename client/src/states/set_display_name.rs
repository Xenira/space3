use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use protocol::protocol::Protocol;
use reqwest::Method;

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::MenuSetDisplayName;
pub(crate) struct SetDisplayNamePlugin;

impl Plugin for SetDisplayNamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisplayName>()
            .add_systems((ui_main_menu, on_network).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource, Default)]
struct DisplayName(String);

fn ui_main_menu(
    mut contexts: EguiContexts,
    mut display_name: ResMut<DisplayName>,
    mut networking: ResMut<NetworkingRessource>,
) {
    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Welcome to the game!");
        ui.separator();
        ui.label(
            "As this is your first time playing, please enter a display name you want to use:",
        );
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.add(egui::TextEdit::singleline(&mut display_name.0));
        });
        ui.horizontal(|ui| {
            if ui.button("Set Display Name").clicked() {
                networking.request_data(Method::PUT, "users/display_name", &display_name.0.clone());
            }
        });
    });
}

fn on_network(
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        if let Protocol::DisplaynameResponse(_) = &ev.0 {
            ev_state_change.send(StateChangeEvent(AppState::MenuMain));
        }
    }
}
