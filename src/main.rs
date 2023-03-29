use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::guard::GuardContext;
use actix_web::http::header;
use futures_util::stream::StreamExt;

use actix_files::NamedFile;
use actix_web::{get, post};
use actix_web::web::{self, Payload};
use actix_web::{App, HttpResponse, HttpServer, HttpRequest, Result};

use mime;

#[get("/")]
async fn hello() -> HttpResponse  {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
r#"<!doctype html>
<html>
    <form action="upload" method="post" enctype="multipart/form-data">
        <label for="fileup">Upload file:</label>
        <input type="file" id="fileup" name="file" accept="*.*">
        <input type="submit" value="Upload" />
    </form>
</html>"#)
}

// GET /uploads -> listing HTML of content of folder uploads/
#[get("/uploads")]
async fn uploads() -> HttpResponse {
    let mut res = String::from("Could not read folder uploads/");
    if let Ok(dir_iter) = Path::new("./uploads/").read_dir() {
        let listing = dir_iter.filter(|r| r.is_ok())
                .map(|r| r.unwrap())
                .map(|de| de.file_name().to_str().map(|f| format!(r#"<li><a href="/files/{f}">{f}</a></li>"#)))
                .filter(|o| o.is_some())
                .fold(String::new(), |mut acc: String, s| { acc.push_str(s.unwrap().as_str()); acc });
        res = format!(r#"<!doctype html>
        <html>
            <h1>Uploads</h1>
            <ul>
                {listing}
            </ul>
        </html>"#);
    }
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(res)
}

// Serves static files
async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path = PathBuf::new();
    path.push("./uploads/");
    path.push(req.match_info().query("filename"));
    Ok(NamedFile::open(path)?)
}

// /upload with a ContentDisposition guard to upload from script :
// curl -X POST --data-binary '@file.txt' -H 'Content-Type: application/octet-stream' -H 'Content-Disposition: attachment; filename="file.txt"' http://127.0.0.1:8000/upload
#[post("/upload", guard = "is_not_form_multipart")]
async fn upload(content_disposition: web::Header<header::ContentDisposition>, mut body: Payload) -> HttpResponse {
    let cd = content_disposition.into_inner();
    if let Some(filename) = cd.get_filename() {
        let mut path = PathBuf::new();
        path.push("./uploads/");
        path.push(filename);
        let mut file = File::create(path).unwrap();
        while let Some(item) = body.next().await {
            file.write_all(&item.unwrap());
        }
        HttpResponse::Created().body(format!("filename: {}", cd.get_filename().unwrap()))
    }
    else {
        HttpResponse::BadRequest().body("Filename is missing")
    }
}

#[derive(MultipartForm)]
struct MultipartUpload {
    file: TempFile,
}

#[post("/upload", guard = "is_form_multipart")]
async fn upload_form(form: MultipartForm<MultipartUpload>) -> HttpResponse {
    let filename = form.file.file_name.as_ref().unwrap();
    let source = form.file.file.path();
    let mut target = PathBuf::from("./uploads/");
    target.push(filename);
    match fs::rename(source, target) {
        Ok(_) => HttpResponse::Created().body(format!("filename: {}", filename)),
        Err(error) => HttpResponse::BadRequest().body(error.to_string())
    }
}

fn is_form_multipart(ctx: &GuardContext) -> bool {
    match ctx.header::<header::ContentType>() {
        Some(ct) => ct.0.type_() == mime::MULTIPART_FORM_DATA.type_(),
        None => false,
    }
}

fn is_not_form_multipart(ctx: &GuardContext) -> bool {
    match ctx.header::<header::ContentType>() {
        Some(ct) => ct.0.type_() != mime::MULTIPART_FORM_DATA.type_(),
        None => false,
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(uploads)
            .route("/files/{filename:.*}", web::get().to(index)) // FIXME regex allows '../'
            .service(upload_form)
            .service(upload)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
