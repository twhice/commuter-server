use rocket::serde::{json, Deserialize, Serialize};
use std::{any::Any, collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(crate = "rocket::serde")]
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserLog {
    pub logs: Vec<Day>,
    pub passwd: String,
}

impl UserLog {
    pub fn new(passwd: String) -> Self {
        Self {
            logs: vec![],
            passwd,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct Logs {
    pub map: HashMap<String, UserLog>,
}

type LogEditor = Box<dyn FnOnce(&mut Logs) -> Box<dyn Any + Send> + Send>;
type Bridge = (mpsc::Sender<LogEditor>, mpsc::Receiver<Box<dyn Any + Send>>);

pub enum LogEditEvent {
    Edit(LogEditor),
    Stop,
}

impl LogEditEvent {
    /// Returns `true` if the log edit event is [`Stop`].
    ///
    /// [`Stop`]: LogEditEvent::Stop
    #[must_use]
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    pub fn try_into_edit(self) -> Result<LogEditor, Self> {
        if let Self::Edit(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

impl<F> From<F> for LogEditEvent
where
    F: FnOnce(&mut Logs) -> Box<dyn Any + Send> + Send + 'static,
{
    fn from(value: F) -> Self {
        Self::Edit(Box::new(value))
    }
}

impl Logs {
    // 日志文件的路径
    pub const LOG_FILE: &'static str = "./logs.json";
    // 自动保存间隔（秒）
    pub const AUTO_SAVE_TIME: u64 = 600;

    async fn new() -> std::io::Result<Self> {
        let logs = Logs::default();
        logs.save().await?;
        Ok(logs)
    }

    async fn save(&self) -> std::io::Result<()> {
        use tokio::fs;
        fs::write(Self::LOG_FILE, json::to_string(self).unwrap())
            .await
            .map(|_| ())
    }
}

async fn start() -> Bridge {
    use tokio::fs;
    let log_file = fs::read_to_string(Logs::LOG_FILE).await;
    let log = match log_file.map(|str| json::from_str::<Logs>(&str).unwrap()) {
        Ok(s) => s,
        Err(_e) => Logs::new().await.unwrap(),
    };
    let log = Arc::new(Mutex::new(log));

    // auto save
    {
        let log = log.clone();
        tokio::task::spawn(async move {
            loop {
                sleep(Duration::from_secs(Logs::AUTO_SAVE_TIME)).await;
                log.lock().await.save().await.unwrap();
            }
        });
    }

    let (event_sender, mut event_reveiver) = mpsc::channel::<LogEditor>(256);
    let (box_sender, box_reveiver) = mpsc::channel::<Box<dyn Any + Send>>(256);

    {
        let log = log.clone();
        tokio::task::spawn(async move {
            loop {
                let editor = event_reveiver.recv().await;
                match editor {
                    Some(editor) => box_sender
                        .send(editor(&mut *log.lock().await))
                        .await
                        .unwrap(),
                    _ => break,
                }
            }
            log.lock().await.save().await.unwrap();
        });
    }
    (event_sender, box_reveiver)
}
async fn bridge<E: Into<LogEditEvent>>(e: E) -> Option<Box<dyn Any + Send>> {
    static mut BRIDGE: Option<Bridge> = None;
    let e: LogEditEvent = e.into();

    unsafe {
        if e.is_stop() {
            BRIDGE = None;
            return None;
        }
        if BRIDGE.is_none() {
            BRIDGE = Some(start().await);
        }
        let _ref = BRIDGE.as_mut().unwrap();
        _ref.0
            .send(e.try_into_edit().unwrap_or_else(|_| unreachable!()))
            .await
            .unwrap();
        _ref.1.recv().await
    }
}

pub async fn edit<R, F>(f: F) -> Option<Box<R>>
where
    R: Any + Send,
    F: FnOnce(&mut Logs) -> R + Send + 'static,
{
    let f = |logs: &mut Logs| Box::new(f(logs)) as Box<dyn Any + Send>;
    let e: LogEditEvent = f.into();
    let r = bridge(e).await;
    r.map(|b| b.downcast().unwrap())
}

pub async fn stop() {
    bridge(LogEditEvent::Stop).await;
}

#[post("/")]
pub async fn route_stop() {
    stop().await;
}
