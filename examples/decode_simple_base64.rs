use std::error::Error;
use aleka::{decode_proto, Message};

fn main() -> Result<(),Box<dyn Error>> {
    env_logger::init();
    let decoded_proto = base64::decode("CgdNaWNoYWVsEBQdcT3qPyIETWlrZSIJVGhlIEJpZyBNKhYKC0Zha2UgU3RyZWV0EgMxMjMaAk5MKhsKDVJhbmRvbSBTdHJlZXQSBjExMjMzNBoCUFQ4xwE=").unwrap();
    let message: Result<Message, Box<dyn Error>> = decode_proto(&decoded_proto);

    let serialized = serde_json::to_string_pretty(&message?).unwrap();
    println!("{}",serialized);

    Ok(())
}