pub mod sockets {

    use std::fs::File;
    use std::io::*;
    use std::net::{TcpListener, TcpStream};
    use snow::{Builder, HandshakeState};
    use crate::stream::stream::{StreamManager};
    
    fn create_noise_responder() -> HandshakeState {
        let builder: Builder<'_> = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse().unwrap());
        let private_key = builder.generate_keypair().unwrap().private;
    
        builder
            .local_private_key(&private_key)
            .build_responder().unwrap()
    }
    
    fn create_noise_initiator() -> HandshakeState {
        let builder: Builder<'_> = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse().unwrap());
        let private_key = builder.generate_keypair().unwrap().private;
    
        builder
            .local_private_key(&private_key)
            .build_initiator().unwrap()
    }
    
    pub fn send(stream: &mut TcpStream, payload: &[u8]) -> () {
        // header is the length of the payload
        // separate 16 bit length into two 8 bit chunks
        // ex.
        // 1111_1111_0101_0101, right shift 8
        // 0000_0000_1111_1111 = header[0]
        // 1111_1111_0101_0101, & 0xff 
        // 0000_0000_0101_0101 = header[1]
        let header = (payload.len() as u16).to_be_bytes();
        stream.write_all(&header).unwrap();
        stream.write_all(payload).unwrap();
    }
    
    pub fn receive(stream: &mut TcpStream) -> Result<Vec<u8>> {
        // read our header
        let mut header = [0u8; 2];
        stream.read_exact(&mut header)?;
    
        let len = ((header[0] as usize) << 8) + header[1] as usize;
        let mut payload = vec![0u8; len];

        stream.read_exact(&mut payload).unwrap();
    
        Ok(payload)
    }
    
    pub fn server_socket(addr: String) -> StreamManager {
        let mut buf = vec![0u8; 65535];
        let (mut stream, _) = TcpListener::bind(addr).unwrap().accept().unwrap();
    
        let mut noise = create_noise_responder();
    
        // receive public key
        noise.read_message(&receive(&mut stream).unwrap(), &mut buf).unwrap();
    
        // send public key & private key
        let len = noise.write_message(&[], &mut buf).unwrap();
        send(&mut stream, &buf[..len]);
    
        // receive private key
        noise.read_message(&receive(&mut stream).unwrap(), &mut buf).unwrap();
    
        let mut noise = noise.into_transport_mode().unwrap();
    
        StreamManager::new(stream, noise)
    }
    
    pub fn client_socket(addr: String) -> StreamManager {
        let mut buf = vec![0u8; 65535];
        let mut stream = TcpStream::connect(addr).unwrap();
    
        let mut noise = create_noise_initiator();
        
        // send public key
        let len = noise.write_message(&[], &mut buf).unwrap();
        send(&mut stream, &buf[..len]);
    
        // receive public key & private key
        noise.read_message(&receive(&mut stream).unwrap(), &mut buf).unwrap();
    
        // send private key
        let len = noise.write_message(&[], &mut buf).unwrap();
        send(&mut stream, &buf[..len]);
    
        let mut noise = noise.into_transport_mode().unwrap();

        StreamManager::new(stream, noise)
    }
}