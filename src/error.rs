use std::{
	error::Error as StdError,
	io::{Error as IoError, ErrorKind},
	ops::{Deref, DerefMut},
};

pub struct Error<ErrorData>
where
	ErrorData: Default,
{
	error: Box<dyn StdError + Send + Sync>,
	status: Option<u16>,
	body: Option<Vec<u8>>,
	data: ErrorData,
}

impl<ErrorData> Error<ErrorData>
where
	ErrorData: Default,
{
	pub fn new(error: Box<dyn StdError + Send + Sync>) -> Error<ErrorData> {
		Error {
			body: None,
			status: None,
			error,
			data: ErrorData::default(),
		}
	}

	pub fn new_with_data(
		error: Box<dyn StdError + Send + Sync>,
		data: ErrorData,
	) -> Self {
		Error {
			error,
			body: None,
			status: None,
			data,
		}
	}

	pub fn get_status(&self) -> Option<u16> {
		self.status
	}

	pub fn status(mut self, status: u16) -> Error<ErrorData> {
		self.status = Some(status);
		self
	}

	pub fn body<TBody>(mut self, body: TBody) -> Self
	where
		TBody: AsRef<str>,
	{
		self.body = Some(body.as_ref().as_bytes().to_vec());
		self
	}

	pub fn get_body_bytes(&self) -> Option<&[u8]> {
		self.body.as_ref().map(AsRef::as_ref)
	}

	pub fn body_bytes(mut self, bytes: &[u8]) -> Self {
		self.body = Some(bytes.to_vec());
		self
	}

	pub fn get_error(&self) -> &Box<dyn StdError + Send + Sync> {
		&self.error
	}

	pub fn get_data(&self) -> &ErrorData {
		&self.data
	}

	pub fn get_data_mut(&mut self) -> &mut ErrorData {
		&mut self.data
	}

	pub fn data(mut self, data: ErrorData) -> Self {
		self.data = data;
		self
	}
}

impl<TErrorData> AsRef<TErrorData> for Error<TErrorData>
where
	TErrorData: Default + Send + Sync,
{
	fn as_ref(&self) -> &TErrorData {
		self.get_data()
	}
}

impl<TErrorData> AsMut<TErrorData> for Error<TErrorData>
where
	TErrorData: Default + Send + Sync,
{
	fn as_mut(&mut self) -> &mut TErrorData {
		self.get_data_mut()
	}
}

impl<TErrorData> Deref for Error<TErrorData>
where
	TErrorData: Default + Send + Sync,
{
	type Target = TErrorData;

	fn deref(&self) -> &TErrorData {
		self.get_data()
	}
}

impl<TErrorData> DerefMut for Error<TErrorData>
where
	TErrorData: Default + Send + Sync,
{
	fn deref_mut(&mut self) -> &mut TErrorData {
		self.get_data_mut()
	}
}

impl<StdErr, TErrorData> From<StdErr> for Error<TErrorData>
where
	StdErr: 'static + StdError + Send + Sync,
	TErrorData: Default + Send + Sync,
{
	fn from(err: StdErr) -> Self {
		Self::new_with_data(Box::new(err), Default::default())
	}
}

pub trait AsError<Value, TErrorData>
where
	Value: Send + Sync,
	TErrorData: Default + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error<TErrorData>>;
	fn body_bytes(self, body: &[u8]) -> Result<Value, Error<TErrorData>>;
	fn body<TBody>(self, body: TBody) -> Result<Value, Error<TErrorData>>
	where
		TBody: AsRef<str>;
}

impl<Value, StdErr, TErrorData> AsError<Value, TErrorData>
	for Result<Value, StdErr>
where
	StdErr: 'static + StdError + Send + Sync,
	Value: Send + Sync,
	TErrorData: Default + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error<TErrorData>> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				Err(Error::new_with_data(Box::new(err), TErrorData::default())
					.status(status))
			}
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error<TErrorData>> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				Err(Error::new_with_data(Box::new(err), TErrorData::default())
					.body_bytes(body))
			}
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, Error<TErrorData>>
	where
		TBody: AsRef<str>,
	{
		match self {
			Ok(value) => Ok(value),
			Err(err) => {
				Err(Error::new_with_data(Box::new(err), TErrorData::default())
					.body(body.as_ref()))
			}
		}
	}
}

impl<Value, TErrorData> AsError<Value, TErrorData>
	for Result<Value, Error<TErrorData>>
where
	Value: Send + Sync,
	TErrorData: Default + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error<TErrorData>> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error<TErrorData>> {
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.body_bytes(body)),
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, Error<TErrorData>>
	where
		TBody: AsRef<str>,
	{
		match self {
			Ok(value) => Ok(value),
			Err(err) => Err(err.body(body.as_ref())),
		}
	}
}

impl<Value, TErrorData> AsError<Value, TErrorData> for Option<Value>
where
	Value: Send + Sync,
	TErrorData: Default + Send + Sync,
{
	fn status(self, status: u16) -> Result<Value, Error<TErrorData>> {
		match self {
			Some(value) => Ok(value),
			None => Err(Error::new_with_data(
				Box::new(IoError::from(ErrorKind::NotFound)),
				TErrorData::default(),
			)
			.status(status)),
		}
	}

	fn body_bytes(self, body: &[u8]) -> Result<Value, Error<TErrorData>> {
		match self {
			Some(value) => Ok(value),
			None => Err(Error::new_with_data(
				Box::new(IoError::from(ErrorKind::NotFound)),
				TErrorData::default(),
			)
			.body_bytes(body)),
		}
	}

	fn body<TBody>(self, body: TBody) -> Result<Value, Error<TErrorData>>
	where
		TBody: AsRef<str>,
	{
		match self {
			Some(value) => Ok(value),
			None => Err(Error::new_with_data(
				Box::new(IoError::from(ErrorKind::NotFound)),
				TErrorData::default(),
			)
			.body(body.as_ref())),
		}
	}
}

pub type DefaultError = Error<()>;
