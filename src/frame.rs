
const PAYLOAD_START: u64 = 112;

pub enum OpCode {
    Continuation,
    Text,
    Binary,
    ConnectionClose,
    Ping,
    Pong
}

/**
*  Read the last 4 bits of the first byte, and set the property according to this:
*  
*  %x0 denotes a continuation frame
*  %x1 denotes a text frame
*  %x2 denotes a binary frame
*  %x3-7 are reserved for further non-control frames
*  %x8 denotes a connection close
*  %x9 denotes a ping
*  %xA denotes a pong
*  %xB-F are reserved for further control frames
*/ 
pub fn opcode(first_byte: u8) -> OpCode {
    let last_four_bits = first_byte % 128;
    match last_four_bits {
        bits if bits == 0 => OpCode::Continuation,
        bits if bits == 1 => OpCode::Text,
        bits if bits == 2 => OpCode::Binary,
        bits if bits == 8 => OpCode::ConnectionClose,
        bits if bits == 9 => OpCode::Ping,
        bits if bits == 10 => OpCode::Pong,
        _ => OpCode::Continuation
    }
}

// TODO CWS: implement masking
pub struct Frame {
    pub fin: bool,
    pub opcode: OpCode,
    pub payload: String,
    pub id: i32
}

impl Frame {
    pub fn new(buffer: &[u8]) -> Result<Self, &'static str> {
        println!("BUFFER: {}", String::from_utf8(buffer.to_vec()).unwrap());
        let first_byte = buffer[0];
        let payload_length_byte = buffer[1];
        let mut fin = false;
        if first_byte > 128  {
            fin = true;
        }

        let payload_length: u64 = match payload_length_byte {
            126 => u16::from_be_bytes([buffer[2], buffer[3]]) as u64,
            127 => u64::from_be_bytes([buffer[2], buffer[3], buffer[4], buffer[5], 
                buffer[6], buffer[7], buffer[8], buffer[9]]),
            _ => payload_length_byte as u64
        };

        let opcode = opcode(first_byte);
        let payload_slice = &buffer[PAYLOAD_START as usize .. (PAYLOAD_START + payload_length) as usize];
        let id = 0;

        return match String::from_utf8(payload_slice.to_vec()) {
            Ok(payload) => {
                Ok(Frame {
                    fin: fin,
                    opcode: opcode,
                    payload: payload,
                    id: id
                })
            },
            Err(e) => {
                println!("{}", e);
                Err("Whoopsies")
            }
            _ => Err("Whoopsies")
        }
    }
}

impl TryFrom<[u8; 128]> for Frame {
    type Error = &'static str;

    fn try_from(value: [u8; 128]) -> Result<Self, Self::Error> {
        Frame::new(&value)
    }
}
