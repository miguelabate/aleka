# Aleka: a schema agnostic protobuf decoder tool

Command line tool and library to decode base64/hex strings of binary protobuf messages
and does its best to decode the contents according to the specification.

The output is in JSON format

https://developers.google.com/protocol-buffers/docs/encoding  
https://developers.google.com/protocol-buffers/docs/encoding#cheat-sheet  

Blog post about with some more details about Aleka [https://miguelabate.com/aleka-a-schema-agnostic-protobuf-decoder/](https://miguelabate.com/aleka-a-schema-agnostic-protobuf-decoder/)  

## Usage

cargo run -- --input-type hex --data 08f4ffffffffffffffff0110d2091a0768656c6c6f6f6f25713d0a422d5e0e0000321e08c80312076d696775656c3112076d696775656c3212076d696775656c3338d30342030f01cf48e58de7ef8c1d50015a1b09bc7f62010000000012076d696775656c3112076d696775656c325a21093f420f0000000000120a6d696775656c63636331120a6d696775656c63636332

## Structure of the output JSON

The output json schema is:
```
//a proto message
Message{
    fields: array of Field
}

//representation of a field of the proto message, it could be a value, or repeated values, or a submessage or a list of submessages.
//Values and message can be filled at the same time in case that the decoding is ambiguous and the result can be both a proto message and a string
Field{
    field_number: int expressing the field number in the proto
    values: array of Value
    messages: array of Message
}

//since we don't have a schema, a value can have different interpretations depending on how it's decoded. That's why there is a list here, to show the different possibilities
Value{
    value_representations: array of ValueRepresentation
}

ValueRepresentation{
    value: the value as a String
    format_type: the type used to represent this value
}
```