mod msg;

use std::sync::Arc;

use axum::{
    extract::{ws::Message, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use dashmap::{DashMap, DashSet};
use futures::{Sink, SinkExt, Stream, StreamExt};
pub use msg::{Msg, MsgData};
use tokio::sync::broadcast;
use tracing::warn;

const CAPACITY: usize = 64;

#[derive(Debug)]
struct State {
    // for a given user, how many rooms they're in
    // user_rooms是一个映射，用于记录每个用户所在的房间数量
    user_rooms: DashMap<String, DashSet<String>>,
    // for a given room, how many users are in it
    // room_users是一个映射，用于记录每个房间内的用户数量
    room_users: DashMap<String, DashSet<String>>,
    // tx是一个消息发送器，用于向其他线程发送消息
    tx: broadcast::Sender<Arc<Msg>>,
}

impl Default for State {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(CAPACITY);
        Self {
            user_rooms: Default::default(),
            room_users: Default::default(),
            tx,
        }
    }
}

// ChatState结构体是State结构体的包装，它通过Arc智能指针来共享State的实例。
#[derive(Debug, Clone, Default)]
pub struct ChatState(Arc<State>);

impl ChatState {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get_user_rooms(&self, username: &str) -> Vec<String> {
        self.0
            .user_rooms
            .get(username)
            .map(|rooms| rooms.clone().into_iter().collect())
            .unwrap_or_default()
    }

    pub fn get_room_users(&self, room: &str) -> Vec<String> {
        self.0
            .room_users
            .get(room)
            .map(|users| users.clone().into_iter().collect())
            .unwrap_or_default()
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    // claims: Claims,
    Extension(state): Extension<ChatState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket<S>(socket: S, state: ChatState)
where
    S: Stream<Item = Result<Message, axum::Error>> + Sink<Message> + Send + 'static,
{
    let mut rx = state.0.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    let state1 = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(msg) => {
                    handle_message(msg.as_str().try_into().unwrap(), state1.0.clone()).await;
                }
                _ => (),
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let data = msg.as_ref().try_into().unwrap();
            if sender.send(Message::Text(data)).await.is_err() {
                warn!("send msg failed");
                break;
            }
        }
    });

    // if any of the tasks fail, we need to shutdown the other one
    tokio::select! {
        _v1 = &mut recv_task => send_task.abort(),
        _v2 = &mut send_task =>  recv_task.abort(),
    }

    // this user has left. Should send a leave message to all rooms
    // usually we can get username from auth header, here we just use "fake_user"
    let username = "fake_user";
    warn!("connection for {username} closed");

    for room in state.get_user_rooms(username) {
        if let Err(e) = state.0.tx.send(Arc::new(Msg::leave(&room, username))) {
            warn!("send leave msg failed: {e}");
        }
    }
}

async fn handle_message(msg: Msg, state: Arc<State>) {
    println!("receive: {msg:?}");
    let msg = match msg.data {
        MsgData::Join => {
            let username = msg.username.clone();
            let room = msg.room.clone();
            state
                .user_rooms
                .entry(username.clone())
                .or_insert_with(DashSet::new)
                .insert(room.clone());
            state
                .room_users
                .entry(room)
                .or_insert_with(DashSet::new)
                .insert(username);
            msg
        }
        MsgData::Leave => {
            if let Some(v) = state.user_rooms.get_mut(&msg.username) {
                v.remove(&msg.room);
                println!("Leave room 1 {msg:?}");
                // v => RwLockGuard
                if v.is_empty() {
                    drop(v);
                    state.user_rooms.remove(&msg.username);
                }
            }
            if let Some(v) = state.room_users.get_mut(&msg.room) {
                v.remove(&msg.username);
                if v.is_empty() {
                    drop(v);
                    state.room_users.remove(&msg.room);
                }
            }
            println!("leave room: {msg:?}");
            msg
        }
        _ => msg,
    };

    if let Err(e) = state.tx.send(Arc::new(msg)) {
        warn!("Failed to send message: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use fake_socket::*;

    #[tokio::test]
    async fn handle_join_should_work() -> Result<()> {
        // let (mut client1, socket1) = create_fake_connection();
        // let (mut client2, socket2) = create_fake_connection();
        // let state = ChatState::new();

        // // mimic server behavior
        // let state1 = state.clone();
        // tokio::spawn(async move {
        //     handle_socket(socket1, state1).await;
        // });

        // let state1 = state.clone();
        // tokio::spawn(async move {
        //     handle_socket(socket2, state1).await;
        // });

        // let msg1 = &Msg::join("room1", "username1");
        // client1.send(Message::Text(msg1.try_into()?))?;

        // let verify = |mut client: FakeClient<Message>,
        //               room: &'static str,
        //               username: &'static str,
        //               data: MsgData| async move {
        //     if let Some(Message::Text(msg1)) = client.recv().await {
        //         let msg = Msg::try_from(msg1.as_str())?;
        //         assert_eq!(msg.room, room);
        //         assert_eq!(msg.username, username);
        //         assert_eq!(msg.data, data);
        //     }
        //     Ok::<_, anyhow::Error>(())
        // };

        let (_client1, _client2, state) = prepare_connections().await?;
        // should first get username1 join msg
        // verify(&mut client1, "room1", "username1", MsgData::Join).await?;
        // verify(&mut client2, "room1", "username1", MsgData::Join).await?;

        // if let Some(Message::Text(msg1)) = client1.recv().await {
        //     let msg = Msg::try_from(msg1.as_str())?;
        //     assert_eq!(msg.room, "room1");
        //     assert_eq!(msg.username, "username1");
        // }

        // if let Some(Message::Text(msg1)) = client2.recv().await {
        //     let msg = Msg::try_from(msg1.as_str())?;
        //     assert_eq!(msg.room, "room1");
        //     assert_eq!(msg.username, "username1");
        // }

        // let msg2 = &Msg::join("room2", "username2");
        // client2.send(Message::Text(msg2.try_into()?))?;

        // then get username join msg
        // assert!(client1.recv().await.is_some());
        // assert!(client2.recv().await.is_some());

        // verify state
        let mut users = state.get_room_users("room1");
        users.sort();
        assert_eq!(users.len(), 2);
        assert_eq!(users, &["username1", "username2"]);

        let rooms = state.get_user_rooms("username1");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms, &["room1"]);

        Ok(())
    }

    #[tokio::test]
    async fn handle_message_and_leave_should_work() -> Result<()> {
        let (mut client1, mut client2, state) = prepare_connections().await?;

        let msg1 = &Msg::new("room1", "username1", MsgData::Message("hello world".into()));
        client1.send(Message::Text(msg1.try_into()?))?;

        verify(
            &mut client1,
            "room1",
            "username1",
            MsgData::Message("hello world".into()),
        )
        .await?;

        verify(
            &mut client2,
            "room1",
            "username1",
            MsgData::Message("hello world".into()),
        )
        .await?;

        let msg2 = &Msg::new("room1", "username1", MsgData::Leave);
        client1.send(Message::Text(msg2.try_into()?))?;

        assert!(client1.recv().await.is_some());
        assert!(client2.recv().await.is_some());

        // verify state
        let mut users = state.get_room_users("room1");
        users.sort();
        assert_eq!(users.len(), 1);
        assert_eq!(users, &["username2"]);

        let rooms = state.get_user_rooms("username1");
        assert!(rooms.is_empty());

        Ok(())
    }

    async fn prepare_connections() -> Result<(FakeClient<Message>, FakeClient<Message>, ChatState)>
    {
        let (mut client1, socket1) = create_fake_connection();
        let (mut client2, socket2) = create_fake_connection();
        let state = ChatState::new();

        // mimic server behavior
        let state1 = state.clone();
        tokio::spawn(async move {
            handle_socket(socket1, state1).await;
        });

        let state1 = state.clone();
        tokio::spawn(async move {
            handle_socket(socket2, state1).await;
        });

        let msg1 = &Msg::join("room1", "username1");
        client1.send(Message::Text(msg1.try_into()?))?;
        let msg2 = &Msg::join("room1", "username2");
        client2.send(Message::Text(msg2.try_into()?))?;

        verify(&mut client1, "room1", "username1", MsgData::Join).await?;
        verify(&mut client2, "room1", "username1", MsgData::Join).await?;

        // then get username join msg
        assert!(client1.recv().await.is_some());
        assert!(client2.recv().await.is_some());

        Ok((client1, client2, state))
    }

    async fn verify(
        client: &mut FakeClient<Message>,
        room: &str,
        username: &str,
        data: MsgData,
    ) -> Result<()> {
        if let Some(Message::Text(msg1)) = client.recv().await {
            let msg = Msg::try_from(msg1.as_str())?;
            assert_eq!(msg.room, room);
            assert_eq!(msg.username, username);
            assert_eq!(msg.data, data);
        }
        Ok::<_, anyhow::Error>(())
    }
}
