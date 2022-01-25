
const PAYLOAD_START: i32 = 112;

enum OpCode {
    Continuation,
    Text,
    Binary,
    ConnectionClose,
    Ping,
    Pong
}

struct Frame {
    fin: bool,
    opcode: OpCode,
    payload: str
}

impl Frame {
    // Parse frame, return a frame with a payload
    pub fn parse(&mut self, &mut buffer: [u8]) {
        let first_byte = buffer[0];
        let payload_length_byte = buffer[1];
        if first_byte > 128  {
            self.fin = true;
        }

        let payload_length = match payload_length_byte {
            126 => u16::from_be_bytes([buffer[2], buffer[3]]),
            127 => u64::from_be_bytes(buffer[2], buffer[3], buffer[4], buffer[5], 
                buffer[6], buffer[7], buffer[8], buffer[9]),
            _ => payload_length_byte
        };

        self.set_opcode(first_byte);
        self.payload = &buffer[PAYLOAD_START..PAYLOAD_START + payload_length];
    }

    fn set_opcode(first_byte: u8) {


    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn can_read_fin_bit() {
        let frame = Frame::new();
        let message: [u8;128] = [];
        frame.parse(message);
        assert_eq!(frame.fin, true);
    }

}