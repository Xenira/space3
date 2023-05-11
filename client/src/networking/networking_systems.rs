use std::sync::Arc;

use async_channel::Receiver;
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use protocol::protocol::{Error, Protocol};
use surf::{Client, Request, Response, StatusCode};

use super::{networking_events::NetworkingEvent, networking_ressource::NetworkingRessource};

#[derive(Component, Debug)]
pub struct NetworkingReceiver(Receiver<NetworkingEvent>);

pub(crate) fn request_dispatcher(
    mut res: ResMut<NetworkingRessource>,
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    let client = Arc::new(res.client.clone());
    for request in res.requests.drain(..) {
        debug!("Spawning task for {:?}", request);
        let task = get_task(&client, request, &thread_pool);

        // Spawn new entity and add our new task as a component
        commands.spawn().insert(NetworkingReceiver(task));
    }
}

fn get_task(
    client: &Client,
    request: Request,
    thread_pool: &Res<AsyncComputeTaskPool>,
) -> Receiver<NetworkingEvent> {
    let c = Arc::new(client.clone());
    let (s, r) = async_channel::unbounded();
    thread_pool
        .spawn(async move {
            debug!("Executing request {:?} with client {:?}", request, c);
            let url = request.url().clone();
            if let Err(err) = s
                .send(NetworkingEvent(match c.send(request).await {
                    Ok(mut response) => decode_response(&mut response, url.path()).await,
                    Err(err) => Error::new_protocol(err.status().into(), err.to_string()),
                }))
                .await
            {
                error!("Failed sending networking event {:?}", err);
            }
        })
        .detach();
    r
}

async fn decode_response(res: &mut Response, path: &str) -> Protocol {
    debug!("Decoding response {:?}", res);

    match res.status() {
        StatusCode::NoContent => Protocol::EMPTY(path.to_string()),
        _ => match res.body_json().await {
            Ok(protocol) => protocol,
            Err(err) => Error::new_protocol(err.status().into(), err.to_string()),
        },
    }
}

pub(crate) fn request_poller(
    mut commands: Commands,
    mut ev: EventWriter<NetworkingEvent>,
    mut transform_tasks: Query<(Entity, &NetworkingReceiver)>,
) {
    for (entity, receiver) in transform_tasks.iter_mut() {
        if let Ok(event) = receiver.0.try_recv() {
            debug!("Sending networking event {:?}", event);
            commands.entity(entity).despawn_recursive();
            ev.send(event);
        } else if receiver.0.is_closed() {
            warn!("Removing entity {:?} with closed receiver. This could indicate networking requests failing.", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}
