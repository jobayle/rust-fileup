use rocket::serde::{Serialize, json::Json};

#[macro_use] extern crate rocket;

#[get("/hw")]
fn hello() -> &'static str {
    "Hello, world!"
}
#[get("/fb")]
fn foo() -> &'static str {
    "FooBar!"
}
#[get("/infos")]
fn infos() -> Json<MyTest> {
    Json(MyTest { id: 1, value: String::from("value") })
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MyTest {
    id: u32,
    value: String
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello,foo,infos])
}
