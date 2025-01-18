use client_lib::sys::open_terminal_with_command;
use client_lib::ClientError;
use client_lib::ClientError::{EnvError, ListenerError};
use std::env::current_exe;
use std::net::{TcpListener, TcpStream};
use std::{env, thread};

pub(crate) fn new_listener() -> Result<TcpListener, ClientError> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(_) => {
            return Err(ListenerError);
        }
    };

    // debug
    let listener_port = match listener.local_addr() {
        Ok(local_addr) => local_addr,
        Err(_) => {
            return Err(ListenerError);
        }
    };
    println!("CLIENT port: {}", listener_port.port());

    Ok(listener)
}

pub(crate) fn start_tui(listener: &TcpListener) -> Result<(), ClientError> {
    env::set_var("RUST_BACKTRACE", "1");

    let listener_port = match listener.local_addr() {
        Ok(local_addr) => local_addr,
        Err(_) => {
            return Err(ListenerError);
        }
    }
    .port();

    let mut tui_exe = match current_exe() {
        Ok(tui_exe) => tui_exe,
        Err(_) => return Err(EnvError),
    };

    tui_exe.pop();
    tui_exe.push("client_tui");
    match open_terminal_with_command(&format!("{} {}", tui_exe.display(), listener_port)) {
        Ok(_) => Ok(()),
        Err(_) => Err(EnvError),
    }
}

pub(crate) fn get_stream(listener: TcpListener) -> Result<TcpStream, ClientError> {
    match listener.accept() {
        Ok((stream, _)) => Ok(stream),
        Err(_) => Err(ListenerError),
    }
}
