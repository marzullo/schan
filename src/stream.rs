pub mod stream {

    use crate::sockets::*;
    use std::net::{TcpStream};
    use snow::*;
    use std::convert::TryFrom;

    pub struct StreamManager {
        buffer: Vec<u8>,
        stream: TcpStream,
        state: TransportState
    }

    impl StreamManager {
        pub fn new(stream: TcpStream, state: TransportState) -> StreamManager {
            StreamManager { buffer: vec![0u8; 65535], stream: stream, state: state }
        }

        pub fn write(&mut self, buffer: &[u8]) -> () {
            let size = 65514;
            let mut totalMessages: usize = (buffer.len() as f32 / size as f32).ceil() as usize;
            let finalMessageSize = buffer.len() % size;

            if totalMessages < 1 {
                totalMessages = 1;
            }

            let mut mbuf = buffer.clone().to_vec();

            // Append the total number of messages in the first 4 bytes of the message
            // Split them accordingly
            for i in 0..totalMessages {
                let mut offset: usize = size;
                let start_index: usize = (i*offset);

                if i == totalMessages-1 {
                    offset = finalMessageSize as usize;
                }

                let mut header: Vec<u8> = (totalMessages as u32).to_be_bytes().to_vec();
                let mut message = Vec::new();
                message.append(&mut header);
                message.append(&mut mbuf[start_index..start_index+offset].to_vec());
                
                let len = self.state.write_message(&message, &mut self.buffer).unwrap();

                sockets::send(&mut self.stream, &self.buffer[..len]);
            }
        } 

        // Read the total amount of messages in the first 4 bytes
        // Chunk them into one big Vec<u8>
        pub fn read(&mut self) -> Result<Vec<u8>, std::io::Error> {
            let mut data = sockets::receive(&mut self.stream)?;
            let mut len = self.state.read_message(&data, &mut self.buffer).unwrap();

            let header = <&[u8;4]>::try_from(&self.buffer[0..4]).unwrap();
            let mc = u32::from_be_bytes(*header);

            let mut output = Vec::new();

            output.append(&mut self.buffer[4..len].to_vec());

            for _ in 1..mc as usize {
                data = sockets::receive(&mut self.stream)?;
                len = self.state.read_message(&data, &mut self.buffer).unwrap();

                output.append(&mut self.buffer[4..len].to_vec());
            }

            Ok(output)
        }
    }

}