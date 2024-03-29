pub mod networking_events;
pub mod networking_ressource;
pub mod networking_systems;
pub mod polling;
pub mod util;

pub mod networking_plugin {
    use bevy::{
        prelude::{debug, App, Plugin, Resource},
        time::Timer,
    };
    use reqwest::Url;

    use crate::networking::polling::RateLimitTimer;

    use super::{
        networking_events::NetworkingEvent,
        networking_ressource::{NetworkingRessource, ServerUrl},
        networking_systems::*,
        polling::{on_polling_status_change, polling_poller, PollingStatus},
    };

    pub struct NetworkingPlugin(pub(crate) String);

    #[derive(Resource)]
    pub struct Runtime(pub tokio::runtime::Runtime);

    impl Plugin for NetworkingPlugin {
        fn build(&self, app: &mut App) {
            debug!("Building networking plugin with base url: {}", self.0);

            #[cfg(not(target_family = "wasm"))]
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            // TODO: Make runtime Option and remove on wasm
            #[cfg(target_family = "wasm")]
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let mut rate_limit_timer = Timer::from_seconds(1.0, bevy::time::TimerMode::Once);
            rate_limit_timer.pause();

            app.add_event::<NetworkingEvent>()
                .insert_resource(ServerUrl(
                    Url::parse(self.0.as_str())
                        .expect("Invalid base url")
                        .join("/api/v1/")
                        .unwrap(),
                ))
                .insert_resource(Runtime(runtime))
                .insert_resource(RateLimitTimer(rate_limit_timer))
                .init_resource::<NetworkingRessource>()
                .add_event::<PollingStatus>()
                .add_system(request_dispatcher)
                .add_system(request_poller)
                .add_system(polling_poller)
                .add_system(on_polling_status_change);
        }
    }
}
