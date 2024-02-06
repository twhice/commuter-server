# 通勤服务器-服务端

## server
为服务器主体

使用

- [`rocket`](https://rocket.rs/)作为http服务器库

- [`tokio`](https://tokio.rs/)提供异步文件IO

## test-client
测试用

- [`reqwest`](https://github.com/seanmonstar/reqwest)和服务器沟通

- [`serde`](https://github.com/serde-rs/serde)和[`serde_json`](https://github.com/serde-rs/json)（反）序列化数据

- [`tokio`](https://tokio.rs/)提供异步运行时

写的确实很垃圾（）， 不过可以从中积累（惨痛的）经验（教训）

> *客户端源码参见[此处](https://github.com/twhice/commuter-site)*

# Build&Run

首先你需要有rust工具链

因为本项目使用了cargo worksapce，你需要使用`--package`指定编译/运行哪个包，比如
```bash
cargo run --package server --release
```
再比如
```bash
cargo run --package test-client 
```

# 配置

`rocket`的配置见[main.rs](./server/src/main.rs)中`main`函数的`config`

更多的配置选项参见[rocket文档](https://docs.rs/rocket/latest/rocket/struct.Config.html)
```rust
let config = rocket::Config {
    address: "0.0.0.0".parse().unwrap(),
    port: 12000,
    ..Default::default()
};
```
- `address`设置为`127.0.0.1`可以使只有本机可以访问服务器，设置为`0.0.0.0`可以使任何设备访问（公开网页）

- `port`为服务器的端口号，此处不多赘述

数据库的配置见[data.rs](./server/src/data.rs)中`Log`结构体的常量
```rust
// 日志文件的路径
pub const LOG_FILE: &'static str = "./logs.json";
// 自动保存间隔（秒）
pub const AUTO_SAVE_TIME: u64 = 600;
```

# LICENSE

MIT