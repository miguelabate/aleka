use clap::Parser;
use clap::ArgEnum;
use log::{info};
use std::error::Error;
use aleka::{decode_proto, Message};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum InputType {
    Hex,
    B64,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input type
    #[clap(short, long, arg_enum, default_value_t = InputType::B64)]
    input_type: InputType,

    /// Input data
    #[clap(short, long)]
    data: String,
}

fn main() -> Result<(),Box<dyn Error>>{ //() is the empty tuple, similar to void, unit type . occupies no memory
    env_logger::init();
    let args = Args::parse();
    info!("Protobuf decoder tool");

    let decoded_proto:Vec<u8> = if args.input_type == InputType::Hex {
        hex::decode(args.data).unwrap()
    }else{
        base64::decode(args.data).unwrap()
    };

    let message: Result<Message, Box<dyn Error>> = decode_proto(&decoded_proto);

    let serialized = serde_json::to_string_pretty(&message?).unwrap();
    println!("{}",serialized);

    Ok(())

}
