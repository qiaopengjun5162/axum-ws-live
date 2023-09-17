# Rust Axum Websocket

## 撰写一个简单的 ChatServer

1. Websocket 消息类型
2. 定义 Msg 类型
3. 创建 axum server
4. 创建 Server State
5. 实现 ws_handler
6. unit test ws_handler

## 实操

```sh
2898  cargo new axum-ws-live
2899  cd axum-ws-live
2900  c
2901  cargo add axum --features ws
2902  cargo add serde --features derive
2903  cargo add serde_json
2904  cargo add tokio --features full
2905  cargo add tracing
2906  cargo add tracing-subscriber
2907  touch src/lib.rs
2908  touch src/msg.rs
2926  cargo add dashmap
2927  cargo add futures
```
