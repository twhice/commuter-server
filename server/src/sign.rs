use crate::data::{self, Day, Logs};
use rocket::serde::{
    json::{self, Json},
    Deserialize, Serialize,
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SignInRequest {
    name: String,
    passwd: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub enum SignInResult {
    IncorrectPassword,
    UserDoesNotExist,
    Success,
    Unknown,
}

/// 登录，同时打卡
#[post("/", data = "<request>")]
pub async fn sign_in(request: Json<SignInRequest>) -> String {
    let sir = data::edit(
        move |logs: &mut Logs| match logs.map.get_mut(&request.name) {
            None => SignInResult::UserDoesNotExist,
            Some(log) => {
                if log.passwd != request.passwd {
                    SignInResult::IncorrectPassword
                } else {
                    let today = Day::today();
                    if !log.logs.last().is_some_and(|day| day == &today) {
                        log.logs.push(today);
                    }
                    SignInResult::Success
                }
            }
        },
    )
    .await
    .unwrap();
    json::to_string(&sir).unwrap()
}
