use std::path::{PathBuf, Path};

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

#[derive(Responder)]
#[response(status = 200, content_type = "text/html; charset=utf-8")]
struct DynHtml(String);

// GET /uploads -> listing HTML of content of folder uploads/
#[get("/uploads")]
fn uploads() -> DynHtml {
    let mut res = String::from("Could not read folder uploads/");
    if let Ok(dir_iter) = Path::new("./uploads/").read_dir() {
        let listing = dir_iter.filter(|r| r.is_ok())
                .map(|r| r.unwrap())
                .map(|de| de.file_name().to_str().map(|f| format!(r#"<li><a href="/files/{}">{}</a></li>"#, f, f)))
                .filter(|o| o.is_some())
                .fold(String::new(), |mut acc: String, s| { acc.push_str(s.unwrap().as_str()); acc });
        res = format!(r#"<!doctype html>
        <html>
            <h1>Uploads</h1>
            <ul>
                {}
            </ul>
        </html>"#, listing);
    }
    DynHtml(res)
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
    rocket::build().mount("/", routes![hello, upload, upload_form, uploads])
        .mount("/files", FileServer::from("./uploads/"))
}
