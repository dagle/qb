use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Convertion error: report upstream")]
    ConvertError,

    #[error("Not a valid command: {0}")]
    InputTypeError(String),
}
