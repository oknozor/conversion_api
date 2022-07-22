#[macro_use] extern crate rocket;

use rocket::serde::{json::Json};
use conversion::Conversion;

mod conversion;

#[post("/convert", data = "<conversion>")]
fn convert(conversion: Json<Conversion>) -> String {
    format!("{}", conversion.execute())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![convert])
}