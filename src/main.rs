use std::path::PathBuf;

use rocket::Data;
use rocket::data::Limits;
use rocket::form::Form;
use rocket::fs::{TempFile, FileServer};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::serde::Serialize;

use crate::headers::*;

pub mod headers;

#[macro_use] extern crate rocket;

#[derive(Responder)]
#[response(status = 200, content_type = "text/html; charset=utf-8")]
struct RawHtml(&'static str);

#[get("/")]
fn hello() -> RawHtml {
    RawHtml(r#"<!doctype html>
    <html>
        <form action="upload" method="post" enctype="multipart/form-data">
            <label for="fileup">Upload file:</label>
            <input type="file" id="fileup" name="file" accept="*.*">
            <input type="submit" value="Upload" />
        </form>
    </html>"#)
}

// /upload with a ContentDisposition guard to upload from script :
// curl -X POST --data-binary '@file.txt' -H 'Content-Type: application/octet-stream' -H 'Content-Disposition: attachment; filename="file.txt"' http://127.0.0.1:8000/upload
#[post("/upload", data = "<data>", rank = 2)]
async fn upload(data: Data<'_>, cd: ContentDisposition, limits: &Limits) -> Result<Status, (Status, std::io::Error)> {
    let mut path = PathBuf::new();
    path.push("./uploads/");
    path.push(cd.content_name);
    match data.open(limits.get("file").unwrap()) // Limit type "file" always defined
        .into_file(path).await {
            Ok(_) => Ok(Status::Created),
            Err(err) => Err((Status::InternalServerError, err))
        }
}

// /upload using form multipart
#[post("/upload", data = "<formfile>")]
async fn upload_form(formfile: Form<TempFile<'_>>) -> Result<Status, (Status, std::io::Error)> {
    let mut file = formfile.into_inner();
    let mut path = PathBuf::new();
    path.push("./uploads/");
    path.push(file.raw_name().map(|fname| fname.dangerous_unsafe_unsanitized_raw().as_str()).unwrap_or("unamed_file"));
    match file.persist_to(path).await {
        Ok(_) => Ok(Status::Created),
        Err(err) => Err((Status::InternalServerError, err))
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MyTest {
    id: u32,
    value: String
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, upload, upload_form])
        .mount("/files", FileServer::from("./uploads/"))
}
