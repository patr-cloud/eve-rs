use std::{
	error::Error as StdError,
	fmt::Debug,
	io::{Error as IoError, ErrorKind},
};

#[derive(Debug)]
pub struct Error {
	pub(crate) body: Option<Vec<u8>>,
	pub(crate) status: Option<u16>,
	pub(crate) error: Box<dyn StdError + Send>,
}

impl Error {
	pub fn new(error: Box<dyn StdError + Send>) -> Self {
		Error {
			body: None,
			status: None,
			error,
		}
	}

	pub fn status(mut self, status: u16) -> Self {
		self.status = Some(status);
		self
	}

	pub fn body(mut self, body: &str) -> Self {
		self.body = Some(body.as_bytes().to_vec());
		self
	}

	pub fn body_bytes(mut self, bytes: &[u8]) -> Self {
		self.body = Some(bytes.to_vec());
		self
	}
}

pub trait AsError<Value>
where
	Value: Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error>;
	fn body_bytes(self, body: &[u8]) -> Result<Value, Error>;
	fn body(self, body: &str) -> Result<Value, Error>;
}

impl<Value> AsError<Value> for Result<Value, Error>
where
	Value: Debug + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.body_bytes(body)),
		}
	}

	fn body(self, body: &str) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.body(body)),
		}
	}
}

impl<Value, StdErr> AsError<Value> for Result<Value, StdErr>
where
	StdErr: 'static + StdError + Send,
	Value: Debug + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(Error::new(Box::new(err)).status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(Error::new(Box::new(err)).body_bytes(body)),
		}
	}

	fn body(self, body: &str) -> Result<Value, Error> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(Error::new(Box::new(err)).body(body)),
		}
	}
}

impl<Value> AsError<Value> for Option<Value>
where
	Value: Debug + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error> {
		match self {
			Some(value) => Ok(value),
			None => {
				Err(Error::new(Box::new(IoError::from(ErrorKind::NotFound)))
					.status(status))
			}
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error> {
		match self {
			Some(value) => Ok(value),
			None => {
				Err(Error::new(Box::new(IoError::from(ErrorKind::NotFound)))
					.body_bytes(body))
			}
		}
	}

	fn body(self, body: &str) -> Result<Value, Error> {
		match self {
			Some(value) => Ok(value),
			None => {
				Err(Error::new(Box::new(IoError::from(ErrorKind::NotFound)))
					.body(body))
			}
		}
	}
}
