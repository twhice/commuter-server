use cors::CORS;

mod cors;
mod data;
mod query;
mod register;
mod sign;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let config = rocket::Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 12000,
        ..Default::default()
    };

    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes![cors::take_cors])
        .mount("/register", routes![register::register])
        .mount("/stop", routes![data::route_stop])
        .mount("/sign_in", routes![sign::sign_in])
        .mount("/query", routes![query::query])
        .configure(config)
        .launch()
        .await?;
    data::stop().await;
    Ok(())
}
