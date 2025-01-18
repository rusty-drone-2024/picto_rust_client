pub mod communication;
pub mod sys;

#[derive(Debug)]
pub enum ClientError {
    ListenerError,
    StreamError,
    EnvError,
    SerializationError,
    LockError,
    TUICommandHandlingError,
    UIError,
}
