#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[get("/api/<req>")]
fn api(req: String) -> String {
    match req.as_str() {
        "health" => String::from("yes"),
        "min_port" => String::from("1337"),
        "max_port" => String::from("1338"),
        "user_queue" => String::from("0"),
        "rate_limit" => String::from("20"),
        "hostname" => String::from("maze.pluxtore.de"),
        "highscore" => String::from("working on it..."),
        "welcome" => String::from("you are not using the official server"),
        _ => String::from("invalid req lul :/"),
    }
}

fn main() {
    rocket::ignite().mount("/", routes![api]).launch();
}
