use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    states::lobby::CurrentLobby,
    AppState, Cleanup, StateChangeEvent,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use protocol::protocol::{LobbyJoinRequest, Protocol};
use reqwest::Method;

const STATE: AppState = AppState::DialogLobbyJoin;
pub(crate) struct DialogLobbyJoinPlugin;

impl Plugin for DialogLobbyJoinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyJoin>()
            .add_systems((ui_lobby_join_dialog, on_join).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource, Default)]
struct LobbyJoin(LobbyJoinRequest);

fn ui_lobby_join_dialog(
    mut contexts: EguiContexts,
    mut network: ResMut<NetworkingRessource>,
    mut lobby: ResMut<LobbyJoin>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Join Lobby");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Lobby-Name:");
            ui.text_edit_singleline(&mut lobby.0.name);
        });
        ui.horizontal(|ui| {
            ui.label("Passphrase:");
            ui.add(egui::TextEdit::singleline(&mut lobby.0.passphrase).password(true));
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Join").clicked() {
                network.request_data(Method::PUT, "lobbies", &lobby.0);
            }
            if ui.button("Cancel").clicked() {
                ev_state_change.send(StateChangeEvent(AppState::MenuMain));
            }
        });
    });
}

fn on_join(
    mut commands: Commands,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        if let Protocol::LobbyStatusResponse(lobby) = &ev.0 {
            debug!("Got lobby response {:?}", lobby);

            commands.insert_resource(CurrentLobby(lobby.clone()));

            ev_state_change.send(StateChangeEvent(AppState::Lobby));
        }
    }
}
