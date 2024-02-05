use rocket::serde::{
    json::{self, Json},
    Deserialize, Serialize,
};

use crate::data::{self, Day};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub enum QueryRequest {
    All,
    SignIned(Day),
    UnSignIned(Day),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Users {
    users: Vec<String>,
}

#[post("/", data = "<request>")]
pub async fn query(request: Json<QueryRequest>) -> String {
    let users = data::edit(move |logs| match request.0 {
        QueryRequest::All => {
            let users = logs.map.keys().cloned().collect();
            Users { users }
        }
        QueryRequest::SignIned(day) => {
            let users = logs
                .map
                .iter()
                .filter_map(|(k, v)| {
                    if v.logs.contains(&day) {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Users { users }
        }
        QueryRequest::UnSignIned(day) => {
            let users = logs
                .map
                .iter()
                .filter_map(|(k, v)| {
                    if !v.logs.contains(&day) {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Users { users }
        }
    })
    .await
    .unwrap();
    json::to_string(&users).unwrap()
}
