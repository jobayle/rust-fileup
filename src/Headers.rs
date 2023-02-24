use std::convert::Infallible;
use std::str::FromStr;

use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use rocket::outcome::Outcome::*;

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition
pub enum ContentTypeEnum {
    Inline,
    Attachment,
    FormData,
}

pub struct ContentDisposition {
    pub content_type: ContentTypeEnum,
    pub content_name: String,
}

impl FromStr for ContentDisposition {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO parse s and construct ContentDisposition
        Ok(ContentDisposition{content_type: ContentTypeEnum::Attachment, content_name: String::from("file.ext")})
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentDisposition {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("Content-Disposition") {
            Some(val) => Success(ContentDisposition::from_str(val).unwrap()),
            None => Failure((Status::BadRequest, "fail"))
        }
    }
}
