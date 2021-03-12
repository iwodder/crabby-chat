use std::net::TcpStream;
use std::borrow::Cow;

//parse room name from raw HTTP headers
pub fn get_room_name(new_stream: &mut TcpStream) -> Option<String> {
    let mut buff = [0; 1024];
    new_stream.peek(&mut buff);
    let incoming = String::from_utf8_lossy(&buff);
    if incoming.starts_with("GET /room/") {
        Some(extract_name(incoming))
    } else {
        None
    }
}

fn extract_name(http_req: Cow<str>) -> String {
    let idx = http_req.find("HTTP");
    let (right, _) = http_req.split_at(idx.unwrap());
    let name_start_idx = right.rfind("/").unwrap() + 1;
    let (_, name) =right.split_at(name_start_idx);
    String::from(name.trim())
}

#[cfg(test)]
mod tests {
    use crate::chat::name_extractor::*;
    #[test]
    fn can_parse_room_name() {
        let s = String::from_utf8_lossy(b"GET /room/hello HTTP/1.1\nHost: 127.0.0.1:8080");
        let name = extract_name(s);
        assert_eq!("hello", name);
    }
}