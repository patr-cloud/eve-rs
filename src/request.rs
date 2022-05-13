use std::{
	collections::HashMap,
	fmt::{Debug, Formatter, Result as FmtResult},
	net::{IpAddr, SocketAddr},
};

use hyper::{
	body::HttpBody,
	header::{self, HeaderName},
	Body,
	Request as HyperRequestInternal,
	Version,
};
use serde::de::DeserializeOwned;

use crate::{error::EveError, HttpMethod};

pub type HyperRequest = HyperRequestInternal<Body>;

pub struct Request {
	pub(crate) socket_addr: SocketAddr,
	pub(crate) hyper_request: HyperRequest,
	pub(crate) method: HttpMethod,
	pub(crate) params: HashMap<String, String>,
	pub(crate) buffered_body: Option<Vec<u8>>,
}

impl Request {
	pub const MAX_REQUEST_LENGTH: usize = 10 * 1024 * 1024; // Max 10 MiB

	pub(crate) fn new(
		socket_addr: SocketAddr,
		method: HttpMethod,
		hyper_request: HyperRequest,
	) -> Self {
		Request {
			socket_addr,
			hyper_request,
			method,
			params: HashMap::new(),
			buffered_body: None,
		}
	}

	pub async fn get_body_bytes(&mut self) -> Result<&[u8], EveError> {
		self.get_body_bytes_with_max_length(Self::MAX_REQUEST_LENGTH)
			.await
	}

	pub async fn get_body_bytes_with_max_length(
		&mut self,
		max_length: usize,
	) -> Result<&[u8], EveError> {
		let body = self.hyper_request.body_mut();
		// If there already is a body that's been buffered, return that.
		let borrow = &mut self.buffered_body;
		if let Some(data) = borrow {
			return Ok(data.as_slice());
		}

		// If there's no body buffered, create a new vec for the variable
		// and read all data onto it until the max limit has been reached
		let bytes = borrow.insert(Vec::with_capacity(
			body.size_hint().upper().unwrap_or_default() as usize,
		));
		loop {
			if bytes.len() >= max_length {
				return Err(EveError::RequestPayloadTooLarge { max_length });
			}
			let data = if let Some(data) = body.data().await {
				// There's more data in the stream. Include it in the buffer
				data
			} else {
				// There's no more data in the stream. Return this value
				break;
			};
			let data = data.map_err(EveError::RequestIo)?;
			bytes.extend_from_slice(&mut data.as_ref());
		}

		Ok(bytes.as_slice())
	}

	pub async fn get_body(&mut self) -> Result<String, EveError> {
		let body = self.get_body_bytes().await?;
		let response = String::from_utf8(body.to_vec())
			.map_err(|_| EveError::RequestBodyInvalidUtf8)?;
		Ok(response)
	}

	pub async fn read_body_chunk(
		&mut self,
	) -> Result<Option<Vec<u8>>, EveError> {
		self.hyper_request
			.body_mut()
			.data()
			.await
			.map(|data| {
				data.map_err(EveError::RequestIo).map(|data| data.to_vec())
			})
			.transpose()
	}

	pub fn get_method(&self) -> &HttpMethod {
		&self.method
	}

	pub fn get_length(&self) -> u128 {
		if let Some(length) = self
			.hyper_request
			.headers()
			.get(header::CONTENT_LENGTH)
			.map(|value| value.to_str().ok())
			.flatten()
		{
			if let Ok(value) = length.parse::<u128>() {
				return value;
			}
		}
		let size_hint = self.hyper_request.body().size_hint();
		size_hint
			.upper()
			.unwrap_or_else(|| size_hint.lower())
			.into()
	}

	pub fn get_path(&self) -> String {
		self.hyper_request.uri().path().to_string()
	}

	pub fn get_full_url(&self) -> String {
		self.hyper_request.uri().to_string()
	}

	pub fn get_origin(&self) -> Option<String> {
		Some(format!(
			"{}://{}",
			self.hyper_request.uri().scheme_str()?,
			self.hyper_request.uri().authority()?
		))
	}

	pub fn get_query(&self) -> &str {
		self.hyper_request.uri().query().unwrap_or("")
	}

	pub fn get_query_as<Q>(&self) -> Result<Q, EveError>
	where
		Q: DeserializeOwned,
	{
		serde_qs::from_str(self.get_query()).map_err(EveError::QueryStringParse)
	}

	pub fn get_host(&self) -> String {
		self.hyper_request
			.uri()
			.host()
			.map(String::from)
			.unwrap_or_else(|| {
				self.get_header(HeaderName::from_static("host"))
					.unwrap_or_else(|| "".to_string())
			})
	}

	pub fn get_host_and_port(&self) -> String {
		format!(
			"{}{}",
			self.hyper_request.uri().host().unwrap(),
			if let Some(port) = self.hyper_request.uri().port_u16() {
				format!(":{}", port)
			} else {
				String::new()
			}
		)
	}

	pub fn get_content_type(&self) -> String {
		let c_type = self
			.get_header(HeaderName::from_static("content-type"))
			.unwrap_or_else(|| "unknown".to_string());
		c_type.split(';').next().unwrap_or("").to_string()
	}

	pub fn get_charset(&self) -> Option<String> {
		let header =
			self.get_header(HeaderName::from_static("content-type"))?;
		let charset_index = header.find("charset=")?;
		let data = &header[charset_index..];
		Some(
			data.chars()
				.skip(charset_index + 8)
				.take(data.find(';').unwrap_or_else(|| data.len()))
				.collect(),
		)
	}

	pub fn get_protocol(&self) -> String {
		// TODO confirm support for X-Forwarded-Proto
		self.hyper_request
			.uri()
			.scheme_str()
			.or(self
				.get_header(HeaderName::from_static("x-forwarded-proto"))
				.as_deref())
			.unwrap_or("http")
			.to_string()
	}

	pub fn is_secure(&self) -> bool {
		self.get_protocol() == "https"
	}

	pub fn get_ip(&self) -> IpAddr {
		self.socket_addr.ip()
	}

	pub fn is(&self, mimes: &[&str]) -> bool {
		let given = self.get_content_type();
		mimes.iter().any(|mime| mime == &given)
	}

	// TODO content negotiation
	// See: https://koajs.com/#request content negotiation

	pub fn get_version(&self) -> &str {
		match self.hyper_request.version() {
			Version::HTTP_09 => "0.9",
			Version::HTTP_10 => "1.0",
			Version::HTTP_11 => "1.1",
			Version::HTTP_2 => "2.0",
			Version::HTTP_3 => "3.0",
			_ => "0.0",
		}
	}

	pub fn get_version_major(&self) -> u8 {
		match self.hyper_request.version() {
			Version::HTTP_09 => 0,
			Version::HTTP_10 | Version::HTTP_11 => 1,
			Version::HTTP_2 => 2,
			Version::HTTP_3 => 3,
			_ => 0,
		}
	}

	pub fn get_version_minor(&self) -> u8 {
		match self.hyper_request.version() {
			Version::HTTP_09 => 9,
			Version::HTTP_10 | Version::HTTP_2 | Version::HTTP_3 => 0,
			Version::HTTP_11 => 1,
			_ => 0,
		}
	}

	pub fn get_header(&self, field: HeaderName) -> Option<String> {
		self.hyper_request
			.headers()
			.iter()
			.find_map(|(key, value)| {
				if key == field {
					value.to_str().map(String::from).ok()
				} else {
					None
				}
			})
	}
	pub fn get_headers(&self) -> HashMap<String, Vec<String>> {
		self.hyper_request
			.headers()
			.iter()
			.filter_map(|(key, value)| {
				Some((key.to_string(), vec![value.to_str().ok()?.to_string()]))
			})
			.collect()
	}

	pub fn get_params(&self) -> &HashMap<String, String> {
		&self.params
	}

	pub fn get_hyper_request(&self) -> &HyperRequest {
		&self.hyper_request
	}

	pub fn get_hyper_request_mut(&mut self) -> &mut HyperRequest {
		&mut self.hyper_request
	}
}

impl Debug for Request {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(
			f,
			"[Request {} {}]",
			self.get_method().to_string(),
			self.get_path()
		)
	}
}
