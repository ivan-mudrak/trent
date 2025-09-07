use crate::types::tx::TxId;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error(Box::new(kind))
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    NotEnoughFunds,
    AccountBlocked,
    TransactionAlreadyUnderDispute,
    NoneExistingTxRef(TxId),
    // 3rd party errors
    Csv(csv::Error),
    Io(std::io::Error),
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match &*self.0 {
            ErrorKind::NotEnoughFunds => None,
            ErrorKind::AccountBlocked => None,
            ErrorKind::TransactionAlreadyUnderDispute => None,
            ErrorKind::NoneExistingTxRef(_) => None,
            ErrorKind::Csv(err) => Some(err),
            ErrorKind::Io(err) => Some(err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self.0 {
            ErrorKind::NotEnoughFunds => write!(f, "{:?}", self.0),
            ErrorKind::AccountBlocked => write!(f, "{:?}", self.0),
            ErrorKind::TransactionAlreadyUnderDispute => write!(f, "{:?}", self.0),
            ErrorKind::NoneExistingTxRef(id) => write!(f, "{:?}: {:?}", self.0, id),
            ErrorKind::Csv(err) => write!(f, "CSV error: {}", err),
            ErrorKind::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::new(ErrorKind::Csv(err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new(ErrorKind::Io(err))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::new(kind)
    }
}
