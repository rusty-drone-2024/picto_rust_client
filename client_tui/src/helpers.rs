use client_lib::ClientError;
use client_lib::ClientError::ListenerError;
use std::env::args;
use std::net::TcpStream;

pub(crate) fn get_stream() -> Result<TcpStream, ClientError> {
    let args: Vec<String> = args().collect();
    let client_port = &*args[1];
    //println!("listener port: {}", client_port);

    let stream = match TcpStream::connect(format!("127.0.0.1:{}", client_port)) {
        Ok(listener) => listener,
        Err(_) => {
            return Err(ListenerError);
        }
    };

    //debug
    if stream.local_addr().is_err() {
        return Err(ListenerError);
    };

    Ok(stream)
}
