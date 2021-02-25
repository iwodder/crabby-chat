use crate::chat::chat_room::Extractor;
use crate::chat::chat_data::{ChatRooms, ChatRoom, ChatUser};

pub mod chat_manager;
mod chat_room;
mod chat_user;


//TBD: Message data definition to go here.
pub mod chat_data {

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RoomCreated {
        pub path: String
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ChatRooms {
        pub rooms: Vec<ChatRoom>
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ChatRoom {
        pub name: String,
        pub users: Vec<ChatUser>
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ChatUser {
        pub name: String
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ChatMessage {
        pub from: String,
        pub msg: String
    }

    #[derive(Serialize,Deserialize, Debug)]
    pub struct RoomAvailable {
        pub name: String,
        pub available: bool
    }
}

pub mod chat_routes {
    use std::sync::Mutex;
    use rocket::State;
    use rocket_contrib::json::Json;

    use crate::chat::chat_data::{ChatRoom, ChatRooms, RoomCreated, RoomAvailable};
    use crate::chat::chat_manager::{ChatManager, Error};

    #[post("/<name>")]
    pub fn create_room(cm: State<Mutex<ChatManager>>, name: String) -> Result<Json<RoomCreated>, Error> {
        let result = cm.lock().unwrap().create_new_room(name.clone());
        match result {
            Ok(_) => {
                Ok(Json(RoomCreated {
                        path: name
                    }))
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    #[get("/")]
    pub fn get_rooms(cm: State<Mutex<ChatManager>>) -> Option<Json<ChatRooms>> {
        let mut res = ChatRooms {
            rooms: vec![]
        };
        for name in cm.lock().unwrap().list_rooms() {
          res.rooms.push(ChatRoom {
             name: name.clone(),
             users: vec![]
          });
        }
        Some(Json(res))
    }

    #[get("/available?<names>")]
    pub fn check_name(cm: State<Mutex<ChatManager>>, names: String) -> Json<Vec<RoomAvailable>> {
        let values:Vec<String> = names.split(",").map(|s|{String::from(s)}).collect();
        let chat_mgr = cm.lock().unwrap();
        let mut response:Vec<RoomAvailable> = vec![];
        for v in values {
            if chat_mgr.name_is_available(&v) {
                response.push(RoomAvailable{
                    name:v,
                    available: true
                });
            } else {
                response.push(RoomAvailable {
                    name: v,
                    available: false
                })
            }
        }
        Json(response)
    }
}

struct JsonExtractor {
    rooms: ChatRooms,
    current_room: Option<ChatRoom>
}

impl JsonExtractor {
    fn new() -> Self {
        JsonExtractor {
            rooms: ChatRooms {
                rooms: vec![]
            },
            current_room: None
        }
    }
}

impl Extractor for JsonExtractor {

    fn pass_name(&mut self, name: String) {
        self.current_room = Some(ChatRoom {
            name,
            users: vec![]
        });
    }

    fn handle_users(&mut self, users: std::slice::Iter<String>) {
        if self.current_room.is_some() {
            let mut room = self.current_room.take().unwrap();

            for user in users {
                room.users.push(ChatUser {
                    name: user.clone()
                });
            }

            self.rooms.rooms.push(room);
            self.current_room = None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::chat::JsonExtractor;
    use crate::chat::chat_room::Extractor;

    #[test]
    fn extractor_starts_with_no_current_room() {
        let extractor = JsonExtractor::new();

        assert!(extractor.current_room.is_none())
    }

    #[test]
    fn passing_name_sets_current_room() {
        let mut extractor = JsonExtractor::new();
        extractor.pass_name(String::from("first-room"));

        assert!(extractor.current_room.is_some())
    }

    #[test]
    fn not_setting_name_does_not_add_to_rooms() {
        let mut extractor = JsonExtractor::new();
        let users = vec![
            String::from("Jim"), String::from("Dwight"), String::from("Andy")];
        extractor.handle_users(users.iter());

        assert_eq!(0, extractor.rooms.rooms.len())
    }

    #[test]
    fn setting_name_adds_users_to_list() {
        let mut extractor = JsonExtractor::new();
        let users = vec![
            String::from("Jim"), String::from("Dwight"), String::from("Andy")];
        extractor.pass_name(String::from("The Office"));
        extractor.handle_users(users.iter());

        assert_eq!(1, extractor.rooms.rooms.len());
        let the_office = extractor.rooms.rooms.pop().unwrap();
        assert_eq!(3, the_office.users.len());
    }

}
