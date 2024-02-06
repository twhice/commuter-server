use rocket::serde::{json, Deserialize, Serialize};
use std::{any::Any, collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};

/// 打卡的时间
///
/// 暂时没去想更好，更合理的方案
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(crate = "rocket::serde")]
pub struct Day {
    pub day: u64,
}

impl Day {
    /// 使用了一个很简单的方法计算打卡的时间是从1970-1-1起的第几天
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

/// 对于单个员工的记录
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserLog {
    /// 哪些天打卡了
    pub logs: Vec<Day>,
    /// 员工的密码
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

/// ”主数据库“
///
/// 就是一个HashMap
#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct Logs {
    /// HashMap<员工的名字,记录>
    pub map: HashMap<String, UserLog>,
}

/// 日志编辑器，是一个被Box包装的函数，可以在线程之间发送
///
/// 函数的返回值是Box<dyn Any + Send>，这是运行时反射，在编辑结束后会被还原为Box<T>
type LogEditor = Box<dyn FnOnce(&mut Logs) -> Box<dyn Any + Send> + Send>;
/// 分别是日志编辑器的发送者，和编辑器返回值的接收者
type Bridge = (mpsc::Sender<LogEditor>, mpsc::Receiver<Box<dyn Any + Send>>);

/// 编辑事件
pub enum LogEditEvent {
    // 编辑日志
    Edit(LogEditor),
    // 停止数据库线程
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

    /// 创建一个新的日志
    async fn new() -> std::io::Result<Self> {
        let logs = Logs::default();
        logs.save().await?;
        Ok(logs)
    }

    /// 保存日志到一个json文件，路径是[Logs::LOG_FILE]
    async fn save(&self) -> std::io::Result<()> {
        use tokio::fs;
        fs::write(Self::LOG_FILE, json::to_string(self).unwrap())
            .await
            .map(|_| ())
    }
}

/// 数据库是否在运行

/// 启动数据库
async fn start() -> Bridge {
    use tokio::fs;
    // 尝试从文件加载
    let log_file = fs::read_to_string(Logs::LOG_FILE).await;
    let log = match log_file.map(|str| json::from_str::<Logs>(&str).unwrap()) {
        Ok(s) => s,
        // 否则新建一个
        Err(_e) => Logs::new().await.unwrap(),
    };
    // 多线程共享log
    let log = Arc::new(Mutex::new(log));
    // 数据库是否在运行
    let running = Arc::new(Mutex::new(true));

    // 自动保存线程
    {
        let log = log.clone();
        let running = running.clone();
        tokio::task::spawn(async move {
            loop {
                // 每隔一段时间自动保存
                sleep(Duration::from_secs(Logs::AUTO_SAVE_TIME)).await;
                // 如果当前的数据库停止了，就不会保存日志并且结束线程，以防止旧的数据库污染新的数据库
                if *running.lock().await {
                    log.lock().await.save().await.unwrap();
                } else {
                    break;
                }
            }
        });
    }

    // 信道大小
    const CHANNEL_SIZE: usize = 128;
    // 编辑者信道
    let (editor_sender, mut editor_reveiver) = mpsc::channel::<LogEditor>(CHANNEL_SIZE);
    // 编辑者的返回值的信道
    let (retv_sender, retv_reveiver) = mpsc::channel::<Box<dyn Any + Send>>(CHANNEL_SIZE);
    {
        let log = log.clone();
        tokio::task::spawn(async move {
            loop {
                let editor = editor_reveiver.recv().await;
                match editor {
                    Some(editor) => retv_sender
                        .send(editor(&mut *log.lock().await))
                        .await
                        .unwrap(),
                    // 如果唯一的editor_sender被drop，就会返回none，此时需要告知自动保存线程不要再保存了
                    _ => {
                        // 告知自动保存线程停止
                        *running.lock().await = false;
                        break;
                    }
                }
            }
            // 不再处理事件，保存日志
            log.lock().await.save().await.unwrap();
        });
    }
    (editor_sender, retv_reveiver)
}

// 和数据库沟通的桥梁
async fn bridge<E: Into<LogEditEvent>>(event: E) -> Option<Box<dyn Any + Send>> {
    // 通过静态变量存储 编辑者的发送者，和编辑者返回值的接收者
    static mut BRIDGE: Option<Bridge> = None;
    let event: LogEditEvent = event.into();

    unsafe {
        // 如果是停止数据库
        if event.is_stop() {
            // 直接销毁Bridge，使它被drop，数据库会自动停止
            BRIDGE = None;
            return None;
        }
        // 如果数据库尚不存在，创建一个
        if BRIDGE.is_none() {
            BRIDGE = Some(start().await);
        }
        // 发送编辑者，接受返回值
        let _ref = BRIDGE.as_mut().unwrap();
        _ref.0
            .send(event.try_into_edit().unwrap_or_else(|_| unreachable!()))
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
    // 进行包装和转化
    let f = |logs: &mut Logs| Box::new(f(logs)) as Box<dyn Any + Send>;
    let r = bridge(f).await;
    // 运行时反射，还原返回值
    r.map(|b| b.downcast().unwrap())
}

pub async fn stop() {
    bridge(LogEditEvent::Stop).await;
}

#[post("/")]
pub async fn route_stop() {
    stop().await;
}
