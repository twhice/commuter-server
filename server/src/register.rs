use crate::data::{self, Logs, UserLog};
use rocket::serde::{
    json::{self, Json},
    Deserialize, Serialize,
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RegisterRequest {
    name: String,
    passwd: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub enum RegisterResult {
    Exist,
    Success,
    Unknown,
}

#[post("/", data = "<request>")]
pub async fn register(request: Json<RegisterRequest>) -> String {
    let rr = data::edit(move |logs: &mut Logs| {
        if logs.map.contains_key(&request.name) {
            return RegisterResult::Exist;
        }
        logs.map
            .insert(request.name.clone(), UserLog::new(request.passwd.clone()));
        RegisterResult::Success
    })
    .await
    .unwrap();
    json::to_string(&rr).unwrap()
}
