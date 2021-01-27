use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Could not bind to acquire public IP")]
    IPError,
}
