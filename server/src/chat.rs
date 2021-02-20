pub mod chat_manager;
mod chat_room;


//TBD: Message data definition to go here.
pub mod chat_data {
    use serde::{Serialize, Deserialize};
    use std::time::Instant;
    use chrono::{DateTime, Utc};

    #[derive(Serialize, Deserialize)]
    pub struct RoomCreated {
        pub path: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct ChatRooms {
        pub rooms: Vec<ChatRoom>
    }

    #[derive(Serialize, Deserialize)]
    pub struct ChatRoom {
        pub name: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct ChatUser {
        pub name: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct ChatMessage {
        pub from: String,
        pub msg: String
    }
}

pub mod chat_routes {
    use rocket::State;
    use crate::chat::chat_manager::ChatManager;
    use std::borrow::BorrowMut;
    use std::sync::Mutex;
    use crate::chat::chat_data;
    use crate::chat::chat_data::{RoomCreated, ChatRooms, ChatRoom};
    use rocket_contrib::json::Json;
    use rocket::http::hyper::StatusCode;


    #[post("/<name>")]
    pub fn create_room(cm: State<Mutex<ChatManager>>, name: String) -> Option<Json<RoomCreated>> {
        let result = cm.lock().unwrap().create_new_room(name.clone());
        if let Ok(_) = result {
            Some(Json(
                RoomCreated {
                    path: name
                }))
        } else {
            None
        }
    }

    #[get("/rooms")]
    pub fn get_rooms(cm: State<Mutex<ChatManager>>) -> Option<Json<ChatRooms>> {
        let rooms = cm.lock().unwrap().list_rooms();
        let mut res = ChatRooms {
            rooms: vec![]
        };
        for name in rooms {
          res.rooms.push(ChatRoom {
             name
          });
        }
        Some(Json(res))
    }
}
