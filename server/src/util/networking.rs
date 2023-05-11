use rocket::{
    request::{self, FromRequest, Outcome},
    Request,
};

pub struct OptionalGuard<T>(Option<T>);

impl<T> OptionalGuard<T> {
    pub fn inner(&self) -> Option<T> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r, T: FromRequest<'r>> FromRequest<'r> for OptionalGuard<T> {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
        if let Outcome::Success(guard) = req.guard::<T>().await {
            return Outcome::Success(OptionalGuard(Some(guard)));
        }

        Outcome::Success(OptionalGuard(None))
    }
}
