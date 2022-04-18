pub mod networking_events;
pub mod networking_ressource;
pub mod networking_systems;

pub mod networking {
    use bevy::prelude::{App, Plugin};
    use surf::Url;

    use super::{
        networking_events::NetworkingEvent,
        networking_ressource::{NetworkingRessource, ServerUrl},
        networking_systems::*,
    };

    pub struct NetworkingPlugin(pub(crate) String);

    impl Plugin for NetworkingPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<NetworkingEvent>()
                .insert_resource(ServerUrl(
                    Url::parse(self.0.as_str()).expect("Invalid base url"),
                ))
                .init_resource::<NetworkingRessource>()
                .add_system(request_dispatcher)
                .add_system(request_poller);
        }
    }
}
