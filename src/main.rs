use std::path::{PathBuf};

use rocket::Data;
use rocket::data::ToByteUnit;
use rocket::serde::{Serialize, json::Json};
use rocket::tokio::fs::File;
use rocket::tokio::io::{BufWriter, BufStream};

use crate::Headers::*;

pub mod Headers;

#[macro_use] extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    r#"<!doctype html>
    <html>
        <form action="upload" method="post">
            <label for="fileup">Upload file:</label>
            <input type="file" id="fileup" name="file" accept="*.*">
        </form>
    </html>"#
}


#[post("/upload", data = "<data>")]
fn upload(data: Data<'_>, cd: ContentDisposition) {
    let mut path = PathBuf::new();
    path.push("./uploads/");
    path.push(cd.content_name);
    data.open(1.gigabytes())
        .stream_to(BufStream::new(File::create(path)));
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MyTest {
    id: u32,
    value: String
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello])
}
