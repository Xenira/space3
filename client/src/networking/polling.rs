use crate::networking::util::get_task;
use async_channel::Receiver;
use bevy::prelude::*;
use protocol::protocol::Protocol;
use reqwest::Method;
use serde::__private::de;

use super::{
    networking_events::NetworkingEvent, networking_plugin::Runtime,
    networking_ressource::NetworkingRessource,
};

#[derive(Component, Debug)]
pub struct PollingReceiver(Receiver<NetworkingEvent>);

#[derive(Resource, Debug)]
pub struct RateLimitTimer(pub Timer);

pub(crate) enum PollingStatus {
    Start,
    Stop,
}

pub(crate) fn on_polling_status_change(
    mut commands: Commands,
    mut ev_polling_status: EventReader<PollingStatus>,
    res: Res<NetworkingRessource>,
    query_poller: Query<Entity, With<PollingReceiver>>,
    runtime: Res<Runtime>,
) {
    for ev in ev_polling_status.iter() {
        match ev {
            PollingStatus::Start => {
                commands.spawn_empty().insert(PollingReceiver(get_task(
                    &runtime,
                    &res.polling_client,
                    res.get_request(Method::GET, "poll").build().unwrap(),
                )));
            }
            PollingStatus::Stop => {
                query_poller
                    .iter()
                    .for_each(|p| commands.entity(p).despawn_recursive());
            }
        };
    }
}

pub(crate) fn polling_poller(
    mut commands: Commands,
    mut ev: EventWriter<NetworkingEvent>,
    transform_tasks: Query<(Entity, &PollingReceiver)>,
    res: Res<NetworkingRessource>,
    runtime: Res<Runtime>,
    mut res_rate_limit_timer: ResMut<RateLimitTimer>,
    time: Res<Time>,
) {
    res_rate_limit_timer.0.tick(time.delta());

    if res_rate_limit_timer.0.finished() {
        res_rate_limit_timer.0.reset();
        res_rate_limit_timer.0.pause();
    }
    if !res_rate_limit_timer.0.paused() {
        return;
    }

    for (entity, receiver) in transform_tasks.iter() {
        if let Ok(event) = receiver.0.try_recv() {
            debug!("Sending networking event {:?}", event);
            if let Protocol::NetworkingError(e) = &event.0 {
                debug!("Networking error, pausing polling: {:?}", e);
                res_rate_limit_timer.0.unpause();
                res_rate_limit_timer.0.reset();
            }

            commands.entity(entity).insert(PollingReceiver(get_task(
                &runtime,
                &res.polling_client,
                res.get_request(Method::GET, "poll").build().unwrap(),
            )));
            ev.send(event);
        } else if receiver.0.is_closed() {
            warn!("Removing entity {:?} with closed receiver. This could indicate networking requests failing.", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}
