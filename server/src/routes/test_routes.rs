use rocket_contrib::json::Json;

#[get("/hello")]
pub fn hello() -> Json<&'static str> {
    Json("hello-world")
}
