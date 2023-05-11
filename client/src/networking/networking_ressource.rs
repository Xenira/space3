use std::time::Duration;

use bevy::prelude::{FromWorld, World};
use protocol::protocol::Protocol;
use serde::Serialize;
use surf::{http::Method, Client, Config, Request, RequestBuilder, Url};

pub struct ServerUrl(pub Url);

impl From<&str> for ServerUrl {
    fn from(base_url: &str) -> Self {
        ServerUrl(Url::parse(base_url).expect("Base url is invalid"))
    }
}

pub struct NetworkingRessource {
    pub client: Client,
    pub requests: Vec<Request>,
}

impl FromWorld for NetworkingRessource {
    fn from_world(world: &mut World) -> Self {
        let base_url = world
            .get_resource::<ServerUrl>()
            .expect("Base path is missing. ServerUrl resource not found");

        NetworkingRessource::new(&base_url.0)
    }
}

impl NetworkingRessource {
    pub fn new(base_url: &Url) -> NetworkingRessource {
        NetworkingRessource {
            client: Config::new()
                .set_base_url(base_url.clone())
                .set_timeout(Some(Duration::from_secs(5)))
                .try_into()
                .expect("Failed to construct client"),
            requests: vec![],
        }
    }

    pub fn request(&mut self, method: Method, url: &str) {
        self.requests.push(self.get_request(method, url).build())
    }

    pub fn request_data<T: Serialize>(&mut self, method: Method, url: &str, data: &T) {
        self.requests.push(
            self.get_request(method, url)
                .body_json(data)
                .expect("Failed to parse json request body")
                .build(),
        )
    }

    pub fn get_request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client.request(method, url)
    }
}
