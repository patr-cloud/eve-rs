use std::{
	convert::TryFrom,
	fmt::{Debug, Formatter, Result as FmtResult},
};

use chrono::Local;
use hyper::{
	body::{Bytes, Sender as HyperSender},
	header::{HeaderName, HeaderValue},
	Body,
	HeaderMap,
};
use tokio::sync::mpsc::UnboundedSender as MpscSender;

#[derive(Debug)]
pub(crate) enum PreBodySenderData {
	Status(u16),
	SetHeader(HeaderName, HeaderValue),
	RemoveHeader(HeaderName),
	ClearHeaders,
	Body(Body),
}

pub(crate) enum ResponseState {
	PreBody {
		header_sender: MpscSender<PreBodySenderData>,
	},
	PostBody {
		body_sender: HyperSender,
	},
}

#[derive(thiserror::Error, Debug)]
pub enum ResponseError {
	#[error("status cannot be set after body as been sent")]
	StatusAfterBody,
	#[error("headers cannot be set after body as been sent")]
	HeaderAfterBody,
	#[error("invalid name `{0}`. Please try to keep your header names to lowercase ASCII characters")]
	InvalidHeaderName(String),
	#[error("invalid value for header name `{0}`. Please try to keep your headers to printable ASCII characters")]
	InvalidHeaderValue(String),
	#[error("an unknown error occured")]
	UnknownError {
		inner: Option<Box<dyn std::error::Error + Send + Sync>>,
	},
}

pub struct Response {
	pub(crate) status: u16,
	pub(crate) headers: HeaderMap,
	pub(crate) response_state: ResponseState,
}

impl Response {
	pub(crate) fn new(header_sender: MpscSender<PreBodySenderData>) -> Self {
		Response {
			status: 200,
			headers: HeaderMap::new(),
			response_state: ResponseState::PreBody { header_sender },
		}
	}

	pub fn get_status(&self) -> u16 {
		self.status
	}
	pub fn get_status_message(&self) -> &str {
		match self.status {
			100 => "continue",
			101 => "switching protocols",
			102 => "processing",
			200 => "ok",
			201 => "created",
			202 => "accepted",
			203 => "non-authoritative information",
			204 => "no content",
			205 => "reset content",
			206 => "partial content",
			207 => "multi-status",
			208 => "already reported",
			226 => "im used",
			300 => "multiple choices",
			301 => "moved permanently",
			302 => "found",
			303 => "see other",
			304 => "not modified",
			305 => "use proxy",
			307 => "temporary redirect",
			308 => "permanent redirect",
			400 => "bad request",
			401 => "unauthorized",
			402 => "payment required",
			403 => "forbidden",
			404 => "not found",
			405 => "method not allowed",
			406 => "not acceptable",
			407 => "proxy authentication required",
			408 => "request timeout",
			409 => "conflict",
			410 => "gone",
			411 => "length required",
			412 => "precondition failed",
			413 => "payload too large",
			414 => "uri too long",
			415 => "unsupported media type",
			416 => "range not satisfiable",
			417 => "expectation failed",
			418 => "I'm a teapot",
			422 => "unprocessable entity",
			423 => "locked",
			424 => "failed dependency",
			426 => "upgrade required",
			428 => "precondition required",
			429 => "too many requests",
			431 => "request header fields too large",
			500 => "internal server error",
			501 => "not implemented",
			502 => "bad gateway",
			503 => "service unavailable",
			504 => "gateway timeout",
			505 => "http version not supported",
			506 => "variant also negotiates",
			507 => "insufficient storage",
			508 => "loop detected",
			510 => "not extended",
			511 => "network authentication required",
			_ => "unknown",
		}
	}
	pub fn set_status(&mut self, code: u16) -> Result<(), ResponseError> {
		if let ResponseState::PreBody { header_sender } =
			&mut self.response_state
		{
			header_sender
				.send(PreBodySenderData::Status(code))
				.map_err(|err| ResponseError::UnknownError {
					inner: Some(Box::new(err)),
				})?;
		} else {
			return Err(ResponseError::StatusAfterBody);
		}
		self.status = code;
		Ok(())
	}

	pub fn set_content_length(
		&mut self,
		length: usize,
	) -> Result<(), ResponseError> {
		self.set_header("content-length", &format!("{}", length))
	}

	pub fn get_header(&self, field: &str) -> Option<String> {
		self.headers
			.get(field)
			.map(|value| value.to_str().ok())
			.flatten()
			.map(String::from)
	}
	pub fn set_header(
		&mut self,
		key: &str,
		value: &str,
	) -> Result<(), ResponseError> {
		if let ResponseState::PreBody { header_sender } =
			&mut self.response_state
		{
			header_sender
				.send(PreBodySenderData::SetHeader(
					HeaderName::try_from(key).map_err(|_| {
						ResponseError::InvalidHeaderName(key.to_string())
					})?,
					HeaderValue::from_str(value).map_err(|_| {
						ResponseError::InvalidHeaderValue(key.to_string())
					})?,
				))
				.map_err(|err| ResponseError::UnknownError {
					inner: Some(Box::new(err)),
				})?;
		} else {
			return Err(ResponseError::HeaderAfterBody);
		}
		Ok(())
	}
	pub fn remove_header(&mut self, key: &str) -> Result<(), ResponseError> {
		if let ResponseState::PreBody { header_sender } =
			&mut self.response_state
		{
			header_sender
				.send(PreBodySenderData::RemoveHeader(
					HeaderName::try_from(key).map_err(|_| {
						ResponseError::InvalidHeaderName(key.to_string())
					})?,
				))
				.map_err(|err| ResponseError::UnknownError {
					inner: Some(Box::new(err)),
				})?;
		} else {
			return Err(ResponseError::HeaderAfterBody);
		}
		Ok(())
	}
	pub async fn clear_headers(&mut self) -> Result<(), ResponseError> {
		if let ResponseState::PreBody { header_sender } =
			&mut self.response_state
		{
			header_sender
				.send(PreBodySenderData::ClearHeaders)
				.map_err(|err| ResponseError::UnknownError {
					inner: Some(Box::new(err)),
				})?;
		} else {
			return Err(ResponseError::HeaderAfterBody);
		}
		Ok(())
	}

	pub fn get_content_type(&self) -> String {
		let c_type = self
			.get_header("Content-Type")
			.unwrap_or_else(|| "text/plain".to_string());
		c_type.split(';').next().unwrap_or("").to_string()
	}
	pub fn set_content_type(
		&mut self,
		content_type: &str,
	) -> Result<(), ResponseError> {
		self.set_header("Content-Type", content_type)
	}

	pub fn redirect(&mut self, url: &str) -> Result<(), ResponseError> {
		self.set_status(302)?;
		self.set_header("Location", url)
	}

	pub fn attachment(
		&mut self,
		file_name: Option<&str>,
	) -> Result<(), ResponseError> {
		self.set_header(
			"Content-Disposition",
			&format!(
				"attachment{}",
				if let Some(filename) = file_name {
					format!("; filename=\"{}\"", filename)
				} else {
					String::new()
				}
			),
		)
	}

	pub fn get_last_modified(&self) -> Option<String> {
		self.get_header("Last-Modified")
	}
	pub fn set_last_modified(
		&mut self,
		last_modified: &str,
	) -> Result<(), ResponseError> {
		self.set_header("Last-Modified", last_modified)
	}

	pub fn set_etag(&mut self, etag: &str) -> Result<(), ResponseError> {
		self.set_header("ETag", etag)
	}

	pub async fn set_body(&mut self, data: &str) -> Result<(), ResponseError> {
		self.set_body_bytes(data.as_bytes()).await
	}
	pub async fn set_body_bytes(
		&mut self,
		data: &[u8],
	) -> Result<(), ResponseError> {
		self.set_content_length(data.len())?;
		self.set_header("date", &Local::now().to_rfc2822())?;

		self.append_body_bytes(data).await
	}
	pub async fn append_body(
		&mut self,
		data: &str,
	) -> Result<(), ResponseError> {
		self.append_body_bytes(data.as_bytes()).await
	}
	pub async fn append_body_bytes(
		&mut self,
		data: &[u8],
	) -> Result<(), ResponseError> {
		match &mut self.response_state {
			ResponseState::PreBody { header_sender } => {
				let (mut sender, body) = Body::channel();
				header_sender.send(PreBodySenderData::Body(body)).map_err(
					|err| ResponseError::UnknownError {
						inner: Some(Box::new(err)),
					},
				)?;
				sender
					.send_data(Bytes::copy_from_slice(data))
					.await
					.map_err(|err| ResponseError::UnknownError {
						inner: Some(Box::new(err)),
					})?;
				self.response_state = ResponseState::PostBody {
					body_sender: sender,
				};
			}
			ResponseState::PostBody { body_sender } => {
				body_sender
					.send_data(Bytes::copy_from_slice(data))
					.await
					.map_err(|err| ResponseError::UnknownError {
						inner: Some(Box::new(err)),
					})?;
			}
		}
		Ok(())
	}

	// TODO
	// pub fn set_cookie(&mut self, cookie: Cookie) {
	// 	self.append_header("Set-Cookie", &cookie.to_header_string());
	// }
}

impl Debug for Response {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "[Response {}]", self.status)
	}
}
