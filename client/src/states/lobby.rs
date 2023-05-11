use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};
use protocol::protocol::{LobbyInfo, Protocol};
use surf::http::Method;

use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup, StateChangeEvent,
};

const STATE: AppState = AppState::Lobby;
pub(crate) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((ui_lobby, on_network).in_set(OnUpdate(STATE)))
            .add_system(cleanup_system::<Cleanup>.in_schedule(OnExit(STATE)));
    }
}

#[derive(Resource)]
pub struct CurrentLobby(pub LobbyInfo);

fn ui_lobby(
    mut contexts: EguiContexts,
    mut network: ResMut<NetworkingRessource>,
    mut lobby: ResMut<CurrentLobby>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("Lobby ({})", lobby.0.name));
        ui.separator();
        for player in &lobby.0.users {
            ui.label(format!(
                "{} ({})",
                player.name,
                if player.ready { "ready" } else { "not ready" }
            ));
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Leave").clicked() {
                network.request_data(Method::Delete, "lobbies", &lobby.0);
            }
            if ui.button("Ready").clicked() {
                network.request_data(Method::Patch, "lobbies/ready", &lobby.0);
            }
            if ui.button("Play").clicked() {
                network.request_data(Method::Patch, "lobbies/start", &lobby.0);
            }
        });
    });
}

fn on_network(
    mut commands: Commands,
    mut ev_networking: EventReader<NetworkingEvent>,
    mut ev_state_change: EventWriter<StateChangeEvent>,
) {
    for ev in ev_networking.iter() {
        match &ev.0 {
            Protocol::LobbyStatusResponse(lobby) => {
                debug!("Got lobby info {:?}", lobby);

                commands.insert_resource(CurrentLobby(lobby.clone()));
            }
            _ => {}
        }
    }
}
