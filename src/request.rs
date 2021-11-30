use std::{
	collections::HashMap,
	fmt::{Debug, Formatter, Result as FmtResult},
	net::{IpAddr, SocketAddr},
};

use hyper::{
	body::HttpBody,
	header,
	Body,
	Request as HyperRequestInternal,
	Version,
};

use crate::HttpMethod;

pub type HyperRequest = HyperRequestInternal<Body>;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
	#[error("error occured while reading from the socket: {0}")]
	Io(hyper::Error),
	#[error("cannot parse body as it exceeded the max length of {max_length}")]
	PayloadTooLarge { max_length: usize },
	#[error("unable to parse body. Unknown format")]
	BodyParse,
	#[error("unable to parse body bytes as utf8")]
	Utf8,
	#[error("unknown method `{0}`")]
	UnknownMethod(String),
}

pub struct Request {
	pub(crate) socket_addr: SocketAddr,
	pub(crate) hyper_request: HyperRequest,
	pub(crate) method: HttpMethod,
	pub(crate) params: HashMap<String, String>,
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
		}
	}

	pub async fn get_body_bytes(&mut self) -> Result<Vec<u8>, RequestError> {
		self.get_body_bytes_with_max_length(Self::MAX_REQUEST_LENGTH)
			.await
	}

	pub async fn get_body_bytes_with_max_length(
		&mut self,
		max_length: usize,
	) -> Result<Vec<u8>, RequestError> {
		let body = self.hyper_request.body_mut();
		let mut bytes = if let Some(length) = body.size_hint().upper() {
			vec![0; length as usize]
		} else {
			vec![]
		};
		while let Some(data) = body.data().await {
			let data = data.map_err(|err| RequestError::Io(err))?;
			if bytes.len() + data.len() >= max_length {
				return Err(RequestError::PayloadTooLarge { max_length });
			}
			bytes.append(&mut data.to_vec());
		}

		Ok(bytes)
	}

	pub async fn get_body(&mut self) -> Result<String, RequestError> {
		let body = self.get_body_bytes().await?;
		let response =
			String::from_utf8(body).map_err(|_| RequestError::Utf8)?;
		Ok(response)
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
		size_hint.upper().unwrap_or(size_hint.lower()).into()
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

	pub fn get_query_string(&self) -> String {
		self.hyper_request.uri().query().unwrap_or("").to_string()
	}

	pub fn get_host(&self) -> String {
		self.hyper_request
			.uri()
			.host()
			.map(String::from)
			.unwrap_or_else(|| {
				self.get_header("host").unwrap_or_else(|| "".to_string())
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
			.get_header("Content-Type")
			.unwrap_or_else(|| "text/plain".to_string());
		c_type.split(';').next().unwrap_or("").to_string()
	}

	pub fn get_charset(&self) -> Option<String> {
		let header = self.get_header("Content-Type")?;
		let charset_index = header.find("charset=")?;
		let data = &header[charset_index..];
		Some(
			data[(charset_index + 8)..
				data.find(';').unwrap_or_else(|| data.len())]
				.to_string(),
		)
	}

	pub fn get_protocol(&self) -> String {
		// TODO confirm support for X-Forwarded-Proto
		self.hyper_request
			.uri()
			.scheme_str()
			.or(self.get_header("X-Forwarded-Proto").as_deref())
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

	pub fn get_version(&self) -> String {
		match self.hyper_request.version() {
			Version::HTTP_09 => "0.9",
			Version::HTTP_10 => "1.0",
			Version::HTTP_11 => "1.1",
			Version::HTTP_2 => "2.0",
			Version::HTTP_3 => "3.0",
			_ => "0.0",
		}
		.to_string()
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

	pub fn get_header(&self, field: &str) -> Option<String> {
		self.hyper_request
			.headers()
			.iter()
			.find_map(|(key, value)| {
				if key.as_str().to_lowercase() == field.to_lowercase() {
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
