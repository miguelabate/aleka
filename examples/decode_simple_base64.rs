use std::error::Error;
use aleka::{decode_proto, Message};

fn main() -> Result<(),Box<dyn Error>> {

    let decoded_proto = base64::decode("CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAAMh4IyAMSB21pZ3VlbDESB21pZ3VlbDISB21pZ3VlbDM40wNCAw8Bz0jljefvjB1QAVobCbx/YgEAAAAAEgdtaWd1ZWwxEgdtaWd1ZWwyWiEJvH9iAQAAAAASCm1pZ3VlbGNjYzESCm1pZ3VlbGNjYzI=").unwrap();
    let message: Result<Message, Box<dyn Error>> = decode_proto(&decoded_proto);

    let serialized = serde_json::to_string_pretty(&message?).unwrap();
    println!("{}",serialized);

    Ok(())
}