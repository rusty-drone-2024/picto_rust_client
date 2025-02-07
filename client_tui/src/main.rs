mod communication;
mod event_handling;
mod helpers;
mod state;
mod ui;

use crate::communication::backend_command_receiver;
use crate::event_handling::handle_event;
use crate::helpers::get_stream;
use crate::state::TUIState;
use crate::ui::ui;
use client_lib::ClientError;
use client_lib::ClientError::{CrossTermError, LockError, StreamError, UIError};
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use ratatui::crossterm::{event, execute};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::cell::RefCell;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::{thread, time};

#[allow(unreachable_code)]
fn main() -> Result<(), ClientError> {
    //INITIALIZE STATE
    let state = Arc::new(Mutex::new(RefCell::new(TUIState::new())));

    //GET TCP CONNECTION TO CLIENT BACKEND
    let stream = get_stream()?;
    let mut client_backend_stream = stream.try_clone().map_err(|_| StreamError)?;

    //BACKEND STATE RECEIVER THREAD
    let state_clone = Arc::clone(&state);
    let stream_clone = client_backend_stream.try_clone().map_err(|_| StreamError)?;
    thread::spawn(move || {
        backend_command_receiver(state_clone, stream_clone);
    });

    //TUI PRE RUN STEPS
    enable_raw_mode().map_err(|_| UIError)?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture).map_err(|_| UIError)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).map_err(|_| UIError)?;

    loop {
        let thirty = time::Duration::from_millis(30);
        sleep(thirty);
        let state = state.lock().map_err(|_| LockError)?;
        let _ = terminal.draw(|frame| ui(frame, state.borrow()));

        let event_available = event::poll(thirty).unwrap();
        if event_available {
            let event = event::read().map_err(|_| UIError)?;
            let _ = handle_event(&mut client_backend_stream, state.borrow_mut(), event);
        }

        let state_borrow = state.borrow();
        let new_title = state_borrow.ui_data.new_window_title.clone();
        drop(state_borrow);

        if state.borrow().ui_data.change_window_title {
            execute!(terminal.backend_mut(), SetTitle(new_title)).map_err(|_| CrossTermError)?;
        }
    }

    //TUI POST RUN STEPS
    disable_raw_mode().map_err(|_| UIError)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|_| UIError)?;
    terminal.show_cursor().map_err(|_| UIError)?;

    //TESTING MESSAGE TO CLIENT
    //send_message(&mut stream, SetName("pippo".to_string()))?;

    Ok(())
}
