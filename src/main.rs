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

/// Examples
/// message MyTest {
///     int32  a_number = 1;
///     int32 another_number = 2;
///     string a_string = 3;
/// }
///
/// CAwQ0gkaB2hlbGxvb28=
/// MyTest{a_number:12,another_number:1234,a_string:"hellooo".to_owned()};
/// CPT//////////wEQ0gkaB2hlbGxvb28=
/// MyTest{a_number:-12,another_number:1234,a_string:"hellooo".to_owned()};
/// CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAA
/// MyTest{a_number:-12,another_number:1234,a_string:"hellooo".to_owned(),a_float:34.56, a_fixed32:3678}
/// CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAAMgMIyAM40wM=
/// MyTest{a_number:-12,another_number:1234,a_string:"hellooo".to_owned(),a_float:34.56, a_fixed32:3678, an_inner: Some(Inner{some_num:456}),a_signed_int:-234};
/// CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAAMh4IyAMSB21pZ3VlbDESB21pZ3VlbDISB21pZ3VlbDM40wM=
/// MyTest{a_number:-12,another_number:1234,a_string:"hellooo".to_owned(),a_float:34.56, a_fixed32:3678, an_inner: Some(Inner{some_num:456, list_of_strings:vec!["miguel1".to_owned(),"miguel2".to_owned(),"miguel3".to_owned() ]}),a_signed_int:-234};
/// CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAAMh4IyAMSB21pZ3VlbDESB21pZ3VlbDISB21pZ3VlbDM40wNCAg8B
///
/// MyTest{a_number:-12,another_number:1234,a_string:"hellooo".to_owned(),a_float:34.56, a_fixed32:3678, an_inner: Some(Inner{some_num:456, list_of_strings:vec!["miguel1".to_owned(),"miguel2".to_owned(),"miguel3".to_owned() ]}) ,a_signed_int:-234, some_bytes:vec![0b0000_1111,0b0000_0001,0b1100_1111]}
/// CPT//////////wEQ0gkaB2hlbGxvb28lcT0KQi1eDgAAMh4IyAMSB21pZ3VlbDESB21pZ3VlbDISB21pZ3VlbDM40wNCAw8Bzw==
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
