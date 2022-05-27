use std::error::Error;
use serde::{Serialize, Deserialize};
use log::{debug, warn};


const TYPE_0:u8 = 0;//Varint-	int32, int64, uint32, uint64, sint32, sint64, bool, enum
const TYPE_1:u8 = 1;//64-bit- fixed64, sfixed64, double
const TYPE_2:u8 = 2;//Length-delimited - string, bytes, embedded messages, packed repeated fields
const TYPE_3:u8 = 3;//groups (deprecated)
const TYPE_4:u8 = 4;//groups (deprecated)
const TYPE_5:u8 = 5;//32-bit- fixed32, sfixed32, float


#[derive(Debug, Deserialize, Serialize)]
pub struct ValueRepresentation {
    pub value: String,
    pub format_type: String,
}

///Represents a value of a field that is not a Message. Since each value can have many decodings depending on the type, we provide a list of possible represntations fo the value.
#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    pub value_representations: Vec<ValueRepresentation>,
}

///Each field of the proto has a field number and can be either a value or another proto message. They are represented as lists becasue can be "repeated"
#[derive(Debug, Deserialize, Serialize)]
pub struct Field {
    pub field_number: i32,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    pub values: Vec<Value>,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    pub messages: Vec<Message>,
}

///Text representation of the proto message
/// A Message has a list of fields.
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub fields: Vec<Field>,
}

impl Message {
    pub fn add_field(&mut self, field:Field){
        for a_field in self.fields.iter_mut() {
            if a_field.field_number == field.field_number {
                field.values.into_iter().for_each(|a_val| {a_field.values.push(a_val)});
                field.messages.into_iter().for_each(|a_msg| {a_field.messages.push(a_msg)});
                return;
            }
        }
        self.fields.push(field);
    }
}

///Represents the encoded key of the proto fields, composed by the field number and its type
pub struct ProtoKey {
    pub field_number:i32,
    pub wire_type:u8,
}

impl ProtoKey {
    pub fn from_varint(a_raw_varint:&[u8]) -> Self {
        let decoded=decode_varint(a_raw_varint);
        let decoded_i32:i32 = decoded.get_value_as_i32();
        ProtoKey{ field_number: decoded_i32 >> 3, wire_type: 0b0000_0111 & (decoded_i32 as u8) }
    }
}

impl ToString for ProtoKey {
    fn to_string(&self) -> String {
        format!("Field number: {} Wire type: {}",self.field_number, self.wire_type)
    }
}

///Contains the bytes of the var int in the reverse order and with the first bit set to 0.
#[derive(Debug)]
pub struct VarIntDecodedData {
    pub value: Vec<u8>,//raw value of the varint bytes, for numbers, the most significant byte to the left
}

impl From<VarIntDecodedData> for i32 {
    fn from(varint_decoded_data: VarIntDecodedData) -> Self {
        varint_decoded_data.get_value_as_i32()
    }
}

impl PartialEq<i32> for VarIntDecodedData {
    fn eq(&self, other: &i32) -> bool {
       *other==self.get_value_as_i32()
    }
}
impl PartialEq<VarIntDecodedData> for i32 {
    fn eq(&self, other: &VarIntDecodedData) -> bool {
        other.get_value_as_i32()==*self
    }
}

impl VarIntDecodedData {
    pub fn get_value_as_i32(&self) -> i32 {
        let mut result:i32 = 0;
        for a_byte in self.value.iter() {
            result<<= 7;
            result |= *a_byte as i32;
        }
        result
    }

    pub fn get_value_as_signed_i32_zigzag(&self) -> i32 {
        let mut result:i32 = 0;
        for a_byte in self.value.iter() {
            result<<= 7;
            result |= *a_byte as i32;
        }
        return (result >> 1) ^ -(result & 1);
    }

    pub fn get_value_as_i64(&self) -> i64 {
        let mut result:i64 = 0;
        for a_byte in self.value.iter() {
            result<<= 7;
            result |= *a_byte as i64;
        }
        result
    }

    pub fn get_value_as_signed_i64_zigzag(&self) -> i64 {
        let mut result:i64 = 0;
        for a_byte in self.value.iter() {
            result<<= 7;
            result |= *a_byte as i64;
        }
        (result >> 1) ^ -(result & 1)
    }
}


pub fn decode_proto(decoded_proto:&[u8]) -> Result<Message, Box<dyn Error>>{
    let mut current_index:usize=0;
    let mut proto_message:Message=Message{ fields: vec![]};

    while current_index<decoded_proto.len() {
        let varint_raw_at = get_varint_raw_at(decoded_proto, current_index);
        let key: ProtoKey = ProtoKey::from_varint(&varint_raw_at);
        current_index += varint_raw_at.len();
        match key.wire_type {
            TYPE_0 => { //int32, int64, uint32, uint64, sint32, sint64, bool, enum
                let (the_varint, offset) = get_varint_decoded_at(&decoded_proto, current_index);
                debug!("Value int32: {}", the_varint.get_value_as_i32());
                debug!("Value zigzag decoded sint32: {}", the_varint.get_value_as_signed_i32_zigzag());
                current_index += offset;
                //fill data in the message object
                proto_message.fields.push(
                    Field{ field_number: key.field_number,
                        values: vec![Value{ value_representations: vec![ValueRepresentation{ value: the_varint.get_value_as_i64().to_string(), format_type: "Varint(int32/int64/uint32/uint64)".to_owned() }, ValueRepresentation{ value: the_varint.get_value_as_signed_i64_zigzag().to_string(), format_type: "Varint(sint32/sint64)".to_owned() }] }],
                        messages: vec![]
                    }
                );
            }
            TYPE_2 => { //string, bytes or subobject
                let (data_length, offset) = get_varint_decoded_at(&decoded_proto, current_index);
                current_index += offset;

                if decoded_proto.len()<current_index ||decoded_proto.len() < current_index + data_length.get_value_as_i32() as usize  {
                    warn!("Index out of bounds");
                    return Err("Index out of bounds".into());
                }

                let raw_data_bytes: &[u8] = &decoded_proto[current_index..current_index+(data_length.get_value_as_i32() as usize)];

                let decoded_sub_message = decode_proto(raw_data_bytes);
                if decoded_sub_message.is_ok() {
                    proto_message.add_field(
                        Field{ field_number: key.field_number,
                            values: vec![],
                            messages: vec![decoded_sub_message?]
                        }
                    );
                }else{//here could be submessage or string of bytes
                    let bytes_to_string = std::str::from_utf8(raw_data_bytes);
                    if bytes_to_string.is_ok() {
                        debug!("String value {}", bytes_to_string.unwrap());
                        proto_message.add_field(
                            Field{ field_number: key.field_number,
                                values: vec![Value{ value_representations: vec![ValueRepresentation{ value: bytes_to_string.unwrap().to_owned(), format_type: "String".to_owned() }] }],
                                messages: vec![]
                            }
                        );
                    }else {
                        //if there was an error probably it' s just bytes
                        proto_message.add_field(
                            Field{ field_number: key.field_number,
                                values: vec![Value{ value_representations: vec![ValueRepresentation{ value: "XXXX".to_owned(), format_type: "Bytes".to_owned() }] }],
                                messages: vec![]
                            }
                        );
                    }

                }


                current_index += data_length.get_value_as_i32()  as usize;
            }
            TYPE_5 => { //expect a chunk of 32 bits (fixed32, sfixed32, float)
                if decoded_proto.len()<current_index ||decoded_proto.len() < current_index + 4 {
                    warn!("Index out of bounds");
                    return Err("Index out of bounds".into());
                }
                let raw_data_bytes: [u8;4] = <[u8; 4]>::try_from(&decoded_proto[current_index..current_index + 4]).unwrap();
                debug!("fixed32 value {}", i32::from_le_bytes(raw_data_bytes));
                debug!("float value {}", f32::from_le_bytes(raw_data_bytes));
                current_index += 4;
                //fill data in the message object
                proto_message.fields.push(
                    Field{ field_number: key.field_number,
                        values: vec![Value{ value_representations: vec![ValueRepresentation{ value: i32::from_le_bytes(raw_data_bytes).to_string(), format_type: "fixed32".to_owned() }, ValueRepresentation{ value: f32::from_le_bytes(raw_data_bytes).to_string(), format_type: "float".to_owned() }] }],
                        messages: vec![]
                    }
                );
            }
            TYPE_1 => { //expect a chunk of fixed 64 bits, fixed64, double(f64) sfixed64
                if decoded_proto.len()<current_index ||decoded_proto.len() < current_index + 8 {
                    warn!("Index out of bounds");
                    return Err("Index out of bounds".into());
                }
                let raw_data_bytes: [u8;8] = <[u8; 8]>::try_from(&decoded_proto[current_index..current_index + 8])?;
                debug!("fixed64 value {}", i64::from_le_bytes(raw_data_bytes));
                debug!("float64 value {}", f64::from_le_bytes(raw_data_bytes));
                current_index += 8;
                //fill data in the message object
                proto_message.fields.push(
                    Field{ field_number: key.field_number,
                        values: vec![Value{ value_representations: vec![ValueRepresentation{ value: i64::from_le_bytes(raw_data_bytes).to_string(), format_type: "fixed64".to_owned() }, ValueRepresentation{ value: f64::from_le_bytes(raw_data_bytes).to_string(), format_type: "double".to_owned() }] }],
                        messages: vec![]
                    }
                );
            }
            TYPE_3 => {
                warn!("Not supported type");
                return Err("Not supported type".into());
            }
            TYPE_4 => {
                warn!("Not supported type");
                return Err("Not supported type".into());
            }
            _ => {
                warn!("Invalid type: {}", key.wire_type);
                return Err("Invalid type".into());
            }
        }
    }
    Ok(proto_message)
}
/** Returns the next varint decoded in the stream plus the amount of bytes read from the input.
It assumes that the inmediate bytes are a varint*/
fn get_varint_decoded_at(a_byte_stream:&[u8], index:usize) -> (VarIntDecodedData, usize) {
    let result:Vec<u8> = get_varint_raw_at(a_byte_stream, index);
    let varint_found= decode_varint(&result);
    (varint_found,result.len())
}

/**Returns the next varint in the stream plus the amount of bytes read from the input.
It assumes that the inmediate bytes are a varint*/
fn get_varint_raw_at(a_byte_stream:&[u8], index:usize) -> Vec<u8> {
    let mut result:Vec<u8> = vec![];
    for i in index .. a_byte_stream.len(){
        if *a_byte_stream.get(i).unwrap() >> 7 == 1 {
            result.push(*a_byte_stream.get(i).unwrap());
        }else{
            result.push(*a_byte_stream.get(i).unwrap());
            break;
        }
    };
    result
}

fn decode_varint(a_varint:&[u8]) -> VarIntDecodedData {
    let mut temp_binarylist:Vec<u8> = vec![];
    // let mut result:i32 = 0;
    //first take out the msb of each
    a_varint.iter().for_each(|a_byte| {
        let new_bte= a_byte & 0b0111_1111;
        // println!("0b{:08b}", new_bte);
        temp_binarylist.push( new_bte);
    });
    temp_binarylist.reverse();//reverse the  groups of 7 bits because varints store numbers with the least significant group first

    let decoded_data: VarIntDecodedData = VarIntDecodedData { value: temp_binarylist };
    decoded_data
}

#[allow(dead_code)]
fn encode_to_varint(a_number:i32) -> Vec<u8> {
    let mut result:Vec<u8> = vec![];

    for i in 0 .. 5 {
        let mut new_byte:u8 = (a_number >> (7*i)) as u8;
        if i==4 {
            new_byte &= 0b0111_1111; // if it's the last byte, mark it with 0 at the beginning
        }else {
            new_byte |= 0b1000_0000;
        }
        // println!("0b{:08b}", new_byte);
        result.push(new_byte);
    }
    result
}

/// cargo test -- --show-output
#[cfg(test)]
mod tests {
    use crate::{decode_varint, VarIntDecodedData, encode_to_varint, get_varint_decoded_at, ProtoKey};

    #[test]
    fn decode_varint_300() {
        let a_varint:Vec<u8> = vec![0b1010_1100, 0b0000_0010];// 300 codified as varint

        let an_int: VarIntDecodedData = decode_varint(&a_varint);
        assert_eq!(300, an_int.get_value_as_i32());
    }

    #[test]
    fn encode_varint_1984() {
        let expected_result:Vec<u8> = vec![0b11000000, 0b10001111, 0b10000000, 0b10000000, 0b00000000];// 1984 codified as varint

        let result:Vec<u8> = encode_to_varint(1984);
        assert_eq!(expected_result, result);
    }

    #[test]
    fn encode_and_then_decode() {
        let an_int2: VarIntDecodedData = decode_varint( &encode_to_varint(-1984));
        assert_eq!(-1984, an_int2.get_value_as_i32());
    }

    #[test]
    fn get_varint_at_different_positions() {
        let mut index:usize = 0;
        let a_list_of_varint:Vec<u8> = vec![0b1010_1100, 0b0000_0010, 0b11000000, 0b10001111, 0b10000000, 0b10000000, 0b00000000,0b00001100,0b11010010,0b00001001];//300, 1984, 12, 1234

        let mut result = get_varint_decoded_at(&a_list_of_varint, index);
        assert_eq!(300, result.0);

        index+=result.1;
        result = get_varint_decoded_at(&a_list_of_varint, index);
        assert_eq!(1984, result.0);

        index+=result.1;
        result = get_varint_decoded_at(&a_list_of_varint, index);
        assert_eq!(12, result.0);

        index+=result.1;
        result = get_varint_decoded_at(&a_list_of_varint, index);
        assert_eq!(1234, result.0);

    }

    #[test]
    fn proto_key_create() {
        let mut a_key_varint:Vec<u8> = vec![0b00001000];
        let mut the_key =ProtoKey::from_varint(&a_key_varint);
        assert_eq!(1, the_key.field_number);
        assert_eq!(0, the_key.wire_type);

        a_key_varint = vec![0b00010000];
        the_key=ProtoKey::from_varint(&a_key_varint);
        assert_eq!(2, the_key.field_number);
        assert_eq!(0, the_key.wire_type);

        a_key_varint = vec![0b00011010];
        the_key=ProtoKey::from_varint(&a_key_varint);
        assert_eq!(3, the_key.field_number);
        assert_eq!(2, the_key.wire_type);
    }
}