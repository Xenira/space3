use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Status,
    request::{FromRequest, Outcome},
    Data, Request, Response,
};
use std::time::SystemTime;

pub struct PerfLogFairing;

#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);

#[rocket::async_trait]
impl Fairing for PerfLogFairing {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Performance Logging",
            kind: Kind::Response | Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        req.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, _: &mut Response<'r>) {
        if request.uri().path() == "/api/v1/poll" {
            return;
        }

        let start = request.local_cache(|| TimerStart(None));

        if let Some(Ok(duration)) = start.0.map(|s| s.elapsed()) {
            match duration.as_millis() {
                0..=100 => debug!("{} took {}ms", request.uri(), duration.as_millis()),
                101..=500 => info!("{} took {}ms", request.uri(), duration.as_millis()),
                501..=1000 => warn!("{} took {}ms", request.uri(), duration.as_millis()),
                _ => error!("{} took {}ms", request.uri(), duration.as_millis()),
            }
        } else {
            debug!("{} took unknown time", request.uri());
        }
    }
}

/// Request guard used to retrieve the start time of a request.
#[derive(Copy, Clone)]
pub struct StartTime(pub SystemTime);

// Allows a route to access the time a request was initiated.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for StartTime {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        match *request.local_cache(|| TimerStart(None)) {
            TimerStart(Some(time)) => Outcome::Success(StartTime(time)),
            TimerStart(None) => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}
