use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Request, Response,
};

/// 用来避免CORS问题，不多赘述
#[allow(clippy::upper_case_acronyms)]
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut Response<'r>) {
        _res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        _res.set_header(Header::new("Access-Control-Allow-Methods", "POST, OPTIONS"));
        _res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        _res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<_..>")]
pub async fn take_cors() -> &'static str {
    ""
}
