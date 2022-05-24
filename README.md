# Aleka: a schema agnostic protobuf decoder tool (WIP)

Command line tool and library to decode base64/hex strings of binary protobuf messages
and does its best to decode the contents according to the specification.

The output is text in JSON format

https://developers.google.com/protocol-buffers/docs/encoding  
https://developers.google.com/protocol-buffers/docs/encoding#cheat-sheet  

## Usage

cargo run --package aleka --bin aleka -- --input-type hex --data 08f4ffffffffffffffff0110d2091a0768656c6c6f6f6f25713d0a422d5e0e0000321e08c80312076d696775656c3112076d696775656c3212076d696775656c3338d30342030f01cf48e58de7ef8c1d50015a1b09bc7f62010000000012076d696775656c3112076d696775656c325a21093f420f0000000000120a6d696775656c63636331120a6d696775656c63636332