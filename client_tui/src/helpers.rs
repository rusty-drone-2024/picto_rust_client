use client_lib::ClientError;
use client_lib::ClientError::ListenerError;
use std::net::TcpStream;

pub(crate) fn get_stream(port: String) -> Result<TcpStream, ClientError> {

    let stream = match TcpStream::connect(format!("127.0.0.1:{}", port)) {
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
