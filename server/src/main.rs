mod data;
mod query;
mod register;
mod sign;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let config = rocket::Config {
        port: 12000,
        ..Default::default()
    };

    let _rocket = rocket::build()
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
