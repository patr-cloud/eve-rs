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
	fn from_error<E: 'static + StdError + Send + Sync>(error: E) -> Self;

	fn status(&mut self, status: u16) -> &mut Self;
	fn body(&mut self, body: impl Into<Vec<u8>>) -> &mut Self;
	fn header(
		&mut self,
		key: impl Into<HeaderName>,
		value: impl Into<HeaderValue>,
	) -> &mut Self;

	fn status_code(&self) -> u16;
	fn body_bytes(&self) -> &[u8];
	fn headers(&self) -> &[(HeaderName, HeaderValue)];
}

pub trait AsError<Value, TError>
where
	Value: Send + Sync,
	TError: Error,
{
	fn status(self, status: u16) -> Result<Value, TError>;
	fn body_bytes(self, body: &[u8]) -> Result<Value, TError>;
	fn body<TBody>(self, body: TBody) -> Result<Value, TError>
	where
		TBody: AsRef<str>;
}

impl<Value, StdErr, TError> AsError<Value, TError> for Result<Value, StdErr>
where
	StdErr: 'static + StdError + Send + Sync,
	Value: Send + Sync,
	TError: Error,
{
	fn status(self, status: u16) -> Result<Value, TError> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				let mut error = TError::from_error(err);
				error.status(status);
				Err(error)
			}
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, TError> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				let mut error = TError::from_error(err);
				error.body(body);
				Err(error)
			}
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, TError>
	where
		TBody: AsRef<str>,
	{
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				let mut error = TError::from_error(err);
				error.body(body.as_ref().as_bytes());
				Err(error)
			}
		}
	}
}

impl<Value, TError> AsError<Value, TError> for Option<Value>
where
	Value: Send + Sync,
	TError: Error,
{
	fn status(self, status: u16) -> Result<Value, TError> {
		match self {
			Some(value) => Ok(value),
			None => {
				let mut err =
					TError::from_msg(format!("expected Some, got None"));
				err.status(status);
				Err(err)
			}
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, TError> {
		match self {
			Some(value) => Ok(value),
			None => {
				let mut err =
					TError::from_msg(format!("expected Some, got None"));
				err.body(body);
				Err(err)
			}
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, TError>
	where
		TBody: AsRef<str>,
	{
		match self {
			Some(value) => Ok(value),
			None => {
				let mut err =
					TError::from_msg(format!("expected Some, got None"));
				err.body(body.as_ref().as_bytes());
				Err(err)
			}
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

pub struct DefaultError {
	status: Option<u16>,
	body: Option<Vec<u8>>,
	headers: Vec<(HeaderName, HeaderValue)>,
	error: Option<EveError>,
}

impl Display for DefaultError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#?}", self.error)
	}
}

impl Default for DefaultError {
	fn default() -> Self {
		Self {
			status: None,
			body: None,
			headers: vec![],
			error: None,
		}
	}
}

const EMPTY_BODY: [u8; 0] = [];

impl Error for DefaultError {
	fn from_msg(message: impl Into<String>) -> Self {
		Self {
			error: Some(EveError::UnknownError(message.into())),
			..Default::default()
		}
	}

	fn from_error<E: 'static + StdError + Send + Sync>(error: E) -> Self {
		Self {
			error: Some(EveError::ServerError(Box::new(error))),
			..Default::default()
		}
	}

	fn status(&mut self, status: u16) -> &mut Self {
		self.status = Some(status);
		self
	}

	fn body(&mut self, body: impl Into<Vec<u8>>) -> &mut Self {
		self.body = Some(body.into());
		self
	}

	fn header(
		&mut self,
		key: impl Into<HeaderName>,
		value: impl Into<HeaderValue>,
	) -> &mut Self {
		self.headers.push((key.into(), value.into()));
		self
	}

	fn status_code(&self) -> u16 {
		self.status.unwrap_or(0)
	}

	fn body_bytes(&self) -> &[u8] {
		self.body.as_deref().unwrap_or(&EMPTY_BODY)
	}

	fn headers(&self) -> &[(HeaderName, HeaderValue)] {
		&self.headers
	}
}
