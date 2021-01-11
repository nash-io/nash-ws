pub type BackendError = crate::backend::Error;

#[derive(Debug)]
pub enum Error {
    ConnectionFailure(BackendError),
    SendError(BackendError),
    ReceiveError(BackendError)
}

pub type WebsocketResult<T> = Result<T, Error>;
