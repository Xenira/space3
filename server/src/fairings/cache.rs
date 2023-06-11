use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{ContentType, Header},
    Request, Response,
};

pub struct CacheFairing;

#[rocket::async_trait]
impl Fairing for CacheFairing {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Cache Control",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if let Some(content_type) = request.content_type() {
            if content_type == &ContentType::HTML {
                response.set_header(Header::new("Cache-Control", "no-cache"));
            } else if content_type == &ContentType::CSS
                || content_type == &ContentType::JavaScript
                || content_type == &ContentType::WASM
            {
                response.set_header(Header::new("Cache-Control", "public, max-age=31536000"));
            }
        }
    }
}
