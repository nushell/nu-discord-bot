#[macro_use] extern crate rocket;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize};
use nu_cli::parse_and_eval;
mod context;
mod run_external;
use context::create_sandboxed_context;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Command<'r> {
    input: &'r str
}


#[post("/", format = "json", data = "<command>")]
fn run_nu(command: Json<Command<'_>>) -> String {
    if let Ok(context) = create_sandboxed_context() {
        match parse_and_eval(command.input, &context) {
            Ok(result) => {
                result
            },
            Err(error) => {
                error.to_string()
            }
        }
    } else {
        "Failed to create a context!".to_string()
    }
    
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    rocket.mount("/", routes![index, run_nu])
}