use std::path::PathBuf;

use nom::error::ErrorKind;
use splr::SolverError;

#[derive(Debug, thiserror::Error)]
pub enum GenericError {
    #[error("Unknown unit specifier: {0}")]
    UnknownSpecifierError(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Wrong token type: {0}")]
    WrongTokenType(String),

    #[error("Nom Error: {0}")]
    NomError(String),

    #[error("Not all tokens were parsed: {0}")]
    SomeTokensWereNotParsed(String),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Traversal error: {0}")]
    IgnoreError(#[from] ignore::Error),

    #[error("Not a file: {0}")]
    NotAFile(PathBuf),

    #[error("Solver error: {0}, statement: {1}")]
    CustomSolverError(SolverError, String)
}

impl GenericError {
    pub fn is_fatal(&self) -> bool {
        match self {
            GenericError::UnknownSpecifierError(_) => true,
            GenericError::UnknownCommand(_) => true,
            GenericError::WrongTokenType(_) => true,
            GenericError::NomError(_) => true,
            GenericError::SomeTokensWereNotParsed(_) => true,
            GenericError::IoError(_) => false,
            GenericError::IgnoreError(_) => false,
            GenericError::NotAFile(_) => false,
            GenericError::CustomSolverError(_, _) => true
        }
    }
}

impl From<GenericError> for nom::Err<nom::error::Error<&str>> {
    fn from(_: GenericError) -> Self {
        let error = nom::error::Error::new("Generic Error", ErrorKind::Alt);
        nom::Err::Error(error)
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for GenericError {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        GenericError::NomError(err.to_string())
    }
}
