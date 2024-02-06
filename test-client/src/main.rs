use reqwest::Client;

use serde::{Deserialize, Serialize};

// const BASE_URL: &str = "http://localhost:8000";
const BASE_URL: &str = "http://154.8.150.125:12000";

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let client = Client::new();
    // let requests = vec![
    //     RegisterRequest {
    //         name: "异月".to_string(),
    //         passwd: "114514".to_string(),
    //     },
    //     RegisterRequest {
    //         name: "550W".to_string(),
    //         passwd: "114514".to_string(),
    //     },
    //     RegisterRequest {
    //         name: "满天都是小星星 护卫队队长".to_string(),
    //         passwd: "863710869".to_string(),
    //     },
    // ];
    // for request in requests {
    //     println!("{:?}", register(&client, request).await?);
    // }

    // let requests = vec![
    //     SignInRequest {
    //         name: "异月".to_string(),
    //         passwd: "114514".to_string(),
    //     },
    //     SignInRequest {
    //         name: "550W".to_string(),
    //         passwd: "114514".to_string(),
    //     },
    // ];

    // for request in requests {
    //     println!("{:?}", sign_in(&client, request).await?);
    // }

    dbg!(get_query(&client, QueryRequest::All).await.unwrap());
    dbg!(get_query(&client, QueryRequest::SignIned(Day::today()))
        .await
        .unwrap());
    dbg!(get_query(&client, QueryRequest::UnSignIned(Day::today()))
        .await
        .unwrap());

    client.post(format!("{BASE_URL}/stop")).send().await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    name: String,
    passwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegisterResult {
    Exist,
    Success,
    Unknown,
}

async fn register(client: &Client, request: RegisterRequest) -> reqwest::Result<RegisterResult> {
    let response = client
        .post(format!("{BASE_URL}/register"))
        .json(&request)
        .send()
        .await?;
    dbg!(response.url());
    Ok(serde_json::from_str::<RegisterResult>(&response.text().await.unwrap()).unwrap())
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SignInRequest {
    name: String,
    passwd: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignInResult {
    IncorrectPassword,
    UserDoesNotExist,
    Success,
    Unknown,
}

async fn sign_in(client: &Client, request: SignInRequest) -> reqwest::Result<SignInResult> {
    let response = client
        .post(format!("{BASE_URL}/sign_in"))
        .json(&request)
        .send()
        .await?;
    Ok(serde_json::from_str::<SignInResult>(&response.text().await.unwrap()).unwrap())
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    pub day: u64,
}

impl Day {
    pub fn today() -> Day {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        const DAY: u64 = 3600 * 24;
        Day { day: t / DAY }
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "D{}", self.day)
    }
}

impl From<u64> for Day {
    fn from(value: u64) -> Self {
        Self { day: value }
    }
}

impl std::ops::Add<u64> for Day {
    type Output = Day;

    fn add(mut self, rhs: u64) -> Self::Output {
        self.day += rhs;
        self
    }
}

impl std::ops::Sub<u64> for Day {
    type Output = Day;

    fn sub(mut self, rhs: u64) -> Self::Output {
        self.day -= rhs;
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryRequest {
    All,
    SignIned(Day),
    UnSignIned(Day),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    users: Vec<String>,
}

async fn get_query(client: &Client, request: QueryRequest) -> reqwest::Result<Users> {
    let response = client
        .post(format!("{BASE_URL}/query"))
        .json(&request)
        .send()
        .await?;
    Ok(serde_json::from_str::<Users>(&response.text().await.unwrap()).unwrap())
}
