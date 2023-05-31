use crate::{
    cleanup_system,
    networking::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource},
    AppState, Cleanup,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use protocol::protocol::{LobbyInfo, Protocol};
use reqwest::Method;

use super::menu_login::User;

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
    lobby: ResMut<CurrentLobby>,
    res_user: Res<User>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("Lobby ({})", lobby.0.name));
        ui.separator();

        let mut ready = false;
        let mut master = lobby.0.master == res_user.0.id;

        for player in &lobby.0.users {
            ui.label(format!(
                "{} ({})",
                player.name,
                if player.ready { "ready" } else { "not ready" }
            ));
            if player.name == *res_user.0.display_name.as_ref().unwrap() {
                ready = player.ready;
                master = player.id == lobby.0.master;
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Leave").clicked() {
                network.request_data(Method::DELETE, "lobbies", &lobby.0);
            }
            if ui
                .button(if !ready { "Ready" } else { "Not Ready" })
                .clicked()
            {
                network.request_data(Method::PATCH, "lobbies/ready", &lobby.0);
            }
            if master {
                if ui.button("Start").clicked() {
                    network.request_data(Method::PATCH, "lobbies/start", &lobby.0);
                }
            }
        });
    });
}

fn on_network(mut commands: Commands, mut ev_networking: EventReader<NetworkingEvent>) {
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
