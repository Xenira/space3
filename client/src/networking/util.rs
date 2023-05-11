use std::sync::Arc;

use async_channel::Receiver;
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use protocol::protocol::{Error, Protocol};
use surf::{Client, Request, Response, StatusCode};

use super::networking_events::NetworkingEvent;

pub(crate) fn get_task(client: &Client, request: Request) -> Receiver<NetworkingEvent> {
    let thread_pool = Arc::new(AsyncComputeTaskPool::get());
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
        StatusCode::Unauthorized => {
            Error::new_protocol(res.status().into(), "Unauthorized".to_string())
        }
        _ => match res.body_json().await {
            Ok(protocol) => protocol,
            Err(err) => Error::new_protocol(err.status().into(), err.to_string()),
        },
    }
}
