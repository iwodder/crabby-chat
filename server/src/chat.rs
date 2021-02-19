pub mod chat_manager;
mod chat_room;


//TBD: Message data definition to go here.
pub mod chat_data {

}

pub mod chat_routes {
    use rocket::State;
    use crate::chat::chat_manager::ChatManager;
    use std::borrow::BorrowMut;
    use std::sync::Mutex;

    #[post("/<name>")]
    pub fn create_room(cm: State<Mutex<ChatManager>>, name: String) -> String {
        let result = cm.lock().unwrap().create_new_room(name);
        //let mgr = cm.create_new_room(name);
        String::from("Room successfully created!!!")
    }
}