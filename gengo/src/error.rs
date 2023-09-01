use std::fmt;
use std::error::Error as ErrorTrait;

macro_rules! error_kind {
    ($($name:ident, $message:literal),*) => {
        /// The kind of error that occurred.
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum ErrorKind {
            $(
                $name,
            )*
        }

        impl fmt::Display for ErrorKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$name => write!(f, $message),
                    )*
                }
            }
        }
    };
}

error_kind!(NoRepository, "no repository found");

impl ErrorTrait for ErrorKind {}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn ErrorTrait>>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub fn with_source<E>(kind: ErrorKind, source: E) -> Self
    where
        E: ErrorTrait + 'static,
    {
        Self {
            kind,
            source: Some(Box::new(source)),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.kind)
    }
}

impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}
