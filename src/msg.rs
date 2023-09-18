use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Msg {
    pub room: String,
    pub username: String,
    pub timestamp: u64,
    pub data: MsgData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsgData {
    Join,
    Leave,
    Message(String),
}

//  `TryFrom`  是 Rust 语言中的一个 trait，用于定义类型之间的转换。
// 它允许从一种类型转换为另一种类型，并在转换过程中处理可能的错误。

//  `TryFrom`  trait 提供了一个  `try_from`  函数，该函数接受一个参数并尝试将其转换为目标类型。
// 如果转换成功，则返回  `Result`  类型的  `Ok`  值，其中包含转换后的目标类型的值。
// 如果转换失败，则返回  `Result`  类型的  `Err`  值，其中包含描述错误的信息。

//  在 Rust 中， `TryFrom`  trait 通常与  `From`  trait 一起使用，用于提供更严格的类型转换。
// `TryFrom`  trait 适用于那些可能会失败的转换，而  `From`  trait 适用于总是能成功转换的情况。
impl TryFrom<&str> for Msg {
    // 这段代码实现了一个  TryFrom<&str>  的 trait for  Msg  结构体。
    // 它将一个字符串  s  转换为  Msg  类型的值，并返回一个结果类型  Result<Self, Self::Error> 。
    type Error = serde_json::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(s)
    }
}

impl TryFrom<&Msg> for String {
    // 这段代码实现了将 &Msg 类型的对象转换为 String 类型的JSON字符串。
    type Error = serde_json::Error;

    fn try_from(msg: &Msg) -> Result<Self, Self::Error> {
        serde_json::to_string(msg)
    }
}

impl Msg {
    pub fn new(room: String, username: String, data: MsgData) -> Self {
        Msg {
            room,
            username,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data,
        }
    }

    /*
    `to_string`  和  `into`  是 Rust 中用于类型转换的两个方法，它们之间有一些区别。

    `to_string`  是一个通用的方法，用于将一个类型转换为  `String`  类型。
    它通常用于将一些可显示的数据类型（如数字、布尔值、字符等）转换为字符串表示形式。

    例如，对于一个整数  `num` ，你可以使用  `num.to_string()`  将其转换为字符串类型。

    `into`  是一个更具体的方法，它是由 Rust 的  `Into`  trait 提供的。
    它允许将一个类型转换为另一个类型，通过将源类型的所有权转移给目标类型。

    例如，如果你有一个  `Vec<i32>`  类型的变量  `vec` ，
    你可以使用  `let new_vec: Vec<i32> = vec.into()`  将其转换为另一个  `Vec<i32>`  类型的变量  `new_vec` 。
    在这个例子中， `into`  方法将转移  `vec`  的所有权到  `new_vec` ，并且你将无法再使用  `vec` 。

    总结来说， `to_string`  用于将类型转换为字符串，而  `into`  则用于将一个类型转换为另一个类型并转移所有权。
     */

    pub fn join(room: &str, username: &str) -> Self {
        Msg::new(room.into(), username.into(), MsgData::Join)
    }

    pub fn leave(room: &str, username: &str) -> Self {
        Msg::new(room.into(), username.into(), MsgData::Leave)
    }

    pub fn message(room: &str, username: &str, message: &str) -> Self {
        Msg::new(
            room.into(),
            username.into(),
            MsgData::Message(message.into()),
        )
    }
}
