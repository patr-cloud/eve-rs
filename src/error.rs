use std::{error::Error as StdError, fmt::Display};

use hyper::header::{HeaderName, HeaderValue};

/*
Requirements of error from Eve:
- Should be able to determine what kind of an error it is (is it an IO error? DB error? etc.)
  Let the user take care of this. Recommended way is to use an enum with thiserror
- Should ideally be able to construct a response from other errors (.status(), .body(), etc)
  Done
- Should be able to create a new error from a string
  Done
- Users should be able to store their own custom data
  Use a struct and impl this Trait
*/

pub trait Error: Display {
	fn from_msg(message: impl Into<String>) -> Self;
	fn from_error<E: StdError + Send + Sync + 'static>(error: E) -> Self;

	fn status(self, status: u16) -> Self;
	fn body(self, body: impl Into<Vec<u8>>) -> Self;
	fn header(
		self,
		key: impl Into<HeaderName>,
		value: impl Into<HeaderValue>,
	) -> Self;

	fn status_code(&self) -> u16;
	fn body_bytes(&self) -> &[u8];
	fn headers(&self) -> &[(HeaderName, HeaderValue)];
}

pub trait AsErrorAble {
	fn into_error(self) -> DefaultError;
}

impl<E: StdError + Send + Sync + 'static> AsErrorAble for E {
	fn into_error(self) -> DefaultError {
		DefaultError::from_error(self)
	}
}

impl AsErrorAble for DefaultError {
	fn into_error(self) -> DefaultError {
		self
	}
}

pub trait AsError<Value>
where
	Value: Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, DefaultError>;
	fn body_bytes(self, body: &[u8]) -> Result<Value, DefaultError>;
	fn body<TBody>(self, body: TBody) -> Result<Value, DefaultError>
	where
		TBody: AsRef<str>;
}

impl<Value, StdErr> AsError<Value> for Result<Value, StdErr>
where
	StdErr: AsErrorAble,
	Value: Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, DefaultError> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.into_error().status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, DefaultError> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.into_error().body(body)),
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, DefaultError>
	where
		TBody: AsRef<str>,
	{
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.into_error().body(body.as_ref().as_bytes())),
		}
	}
}

impl<Value> AsError<Value> for Option<Value>
where
	Value: Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, DefaultError> {
		match self {
			Some(value) => Ok(value),
			None => Err(DefaultError::from_msg(
				"expected Some, got None".to_string(),
			)
			.status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, DefaultError> {
		match self {
			Some(value) => Ok(value),
			None => Err(DefaultError::from_msg(
				"expected Some, got None".to_string(),
			)
			.body(body)),
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, DefaultError>
	where
		TBody: AsRef<str>,
	{
		match self {
			Some(value) => Ok(value),
			None => Err(DefaultError::from_msg(
				"expected Some, got None".to_string(),
			)
			.body(body.as_ref().as_bytes())),
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum EveError {
	// Request errors
	#[error("error occured while reading from the socket: {0}")]
	RequestIo(hyper::Error),
	#[error("cannot parse request body as it exceeded the max length of {max_length}")]
	RequestPayloadTooLarge { max_length: usize },
	#[error("unable to parse query string in the requested format")]
	QueryStringParse(#[from] serde_qs::Error),
	#[error("unable to parse request body. Unknown format")]
	RequestBodyInvalidFormat,
	#[error("unable to parse request body bytes as utf8")]
	RequestBodyInvalidUtf8,
	#[error("unknown HTTP method `{0}`")]
	UnknownHttpMethod(String),

	// Response errors
	#[error("error occured while writing to the socket: {0}")]
	ResponseIo(hyper::Error),
	#[error("status cannot be set after body as been sent")]
	StatusSetAfterBody,
	#[error("headers cannot be set after body as been sent")]
	HeaderSetAfterBody,
	#[error("invalid header name `{0}`. Please try to keep your header names to lowercase ASCII characters")]
	InvalidResponseHeaderName(String),
	#[error("invalid value for header name `{0}`. Please try to keep your headers to printable ASCII characters")]
	InvalidResponseHeaderValue(String),

	// Misc errors
	#[error("unknown error: {0}")]
	UnknownError(String),
	#[error("unknown internal server error: {0}")]
	ServerError(Box<dyn StdError + Send + Sync>),
}

#[derive(Debug)]
pub struct DefaultError {
	status: Option<u16>,
	body: Option<Vec<u8>>,
	headers: Vec<(HeaderName, HeaderValue)>,
	error: EveError,
}

impl DefaultError {
	pub fn empty() -> Self {
		Self {
			error: EveError::UnknownError("unknown error".into()),
			status: None,
			body: None,
			headers: vec![],
		}
	}

	pub fn as_result<TSuccess>() -> Result<TSuccess, Self> {
		Err(Self::empty())
	}
}

impl Display for DefaultError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"DefaultError: {}. Status: {}",
			self.error,
			self.status
				.map(|status| status.to_string())
				.as_deref()
				.unwrap_or("-")
		)
	}
}

impl Error for DefaultError {
	fn from_msg(message: impl Into<String>) -> Self {
		Self {
			error: EveError::UnknownError(message.into()),
			status: None,
			body: None,
			headers: vec![],
		}
	}

	fn from_error<E: 'static + StdError + Send + Sync>(error: E) -> Self {
		Self {
			error: EveError::ServerError(Box::new(error)),
			status: None,
			body: None,
			headers: vec![],
		}
	}

	fn status(mut self, status: u16) -> Self {
		self.status = Some(status);
		self
	}

	fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
		self.body = Some(body.into());
		self
	}

	fn header(
		mut self,
		key: impl Into<HeaderName>,
		value: impl Into<HeaderValue>,
	) -> Self {
		self.headers.push((key.into(), value.into()));
		self
	}

	fn status_code(&self) -> u16 {
		self.status.unwrap_or(500)
	}

	fn body_bytes(&self) -> &[u8] {
		self.body.as_deref().unwrap_or(&[])
	}

	fn headers(&self) -> &[(HeaderName, HeaderValue)] {
		&self.headers
	}
}

impl<E: StdError + Send + Sync + 'static> From<E> for DefaultError {
	fn from(error: E) -> Self {
		Self::from_error(error)
	}
}
