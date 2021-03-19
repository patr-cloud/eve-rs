use crate::{cookie::Cookie, file_uploader::Field, HttpMethod};
use hyper::{body::HttpBody, Body, Request as HyperRequestInternal, Uri, Version};
use std::{
	collections::HashMap,
	fmt::{Debug, Formatter, Result as FmtResult},
	net::{IpAddr, SocketAddr},
	str,
	sync::Arc,
};
use tokio::sync::Mutex;

pub type HyperRequest = Arc<Mutex<HyperRequestInternal<Body>>>;

pub struct Request {
	pub(crate) socket_addr: SocketAddr,
	pub(crate) body: Option<Vec<u8>>,
	pub(crate) method: HttpMethod,
	pub(crate) uri: Uri,
	pub(crate) version: (u8, u8),
	pub(crate) headers: HashMap<String, Vec<String>>,
	pub(crate) query: HashMap<String, String>,
	pub(crate) params: HashMap<String, String>,
	pub(crate) cookies: Vec<Cookie>,
	pub(crate) hyper_request: HyperRequest,
	pub(crate) max_request_body: usize,
}

impl Request {
	pub async fn from_hyper(socket_addr: SocketAddr, req: HyperRequestInternal<Body>) -> Self {
		let (parts, hyper_body) = req.into_parts();
		let mut headers = HashMap::<String, Vec<String>>::new();
		parts.headers.iter().for_each(|(key, value)| {
			let key = key.to_string();
			let value = value.to_str();

			if value.is_err() {
				return;
			}
			let value = value.unwrap().to_string();

			if let Some(values) = headers.get_mut(&key) {
				values.push(value);
			} else {
				headers.insert(key.to_string(), vec![value]);
			}
		});
		Request {
			socket_addr,
			body: None,
			method: HttpMethod::from(parts.method.clone()),
			uri: parts.uri.clone(),
			version: match parts.version {
				Version::HTTP_09 => (0, 9),
				Version::HTTP_10 => (1, 0),
				Version::HTTP_11 => (1, 1),
				Version::HTTP_2 => (2, 0),
				Version::HTTP_3 => (3, 0),
				_ => (0, 0),
			},
			headers: headers.clone(),
			query: if let Some(query) = parts.uri.query() {
				serde_urlencoded::from_str(query).unwrap_or_default()
			} else {
				HashMap::new()
			},
			params: HashMap::new(),
			cookies: vec![],
			hyper_request: Arc::new(Mutex::new(HyperRequestInternal::from_parts(
				parts, hyper_body,
			))),
			max_request_body: usize::MAX,
		}
	}

	pub async fn get_body_bytes(&mut self) -> Option<&[u8]> {
		if self.body.is_none() {
			let mut body = vec![];
			while let Some(data) = self.hyper_request.lock().await.data().await {
				if data.is_err() {
					return None;
				}
				body.append(&mut data.unwrap().to_vec());
				if body.len() >= self.max_request_body {
					return None;
				}
			}

			self.body = Some(body);
		}

		self.body.as_ref().map(|body| body.as_ref())
	}

	pub async fn get_body(&mut self) -> Option<String> {
		let body_bytes = self.get_body_bytes().await?;
		let result = str::from_utf8(&body_bytes);
		if result.is_err() {
			return None;
		}
		let result = result.unwrap();
		Some(result.to_string())
	}

	pub fn get_method(&self) -> &HttpMethod {
		&self.method
	}

	pub fn get_length(&self) -> u128 {
		if let Some(length) = self.headers.get("Content-Length") {
			if let Some(value) = length.get(0) {
				if let Ok(value) = value.parse::<u128>() {
					return value;
				}
			}
		}
		0
	}

	pub fn get_path(&self) -> String {
		self.uri.path().to_string()
	}

	pub fn get_full_url(&self) -> String {
		self.uri.to_string()
	}

	pub fn get_origin(&self) -> Option<String> {
		Some(format!(
			"{}://{}",
			self.uri.scheme_str()?,
			self.uri.authority()?
		))
	}

	pub fn get_query_string(&self) -> String {
		self.uri.query().unwrap_or("").to_string()
	}

	pub fn get_host(&self) -> String {
		self.uri
			.host()
			.map(String::from)
			.unwrap_or_else(|| self.get_header("host").unwrap_or_else(|| "".to_string()))
	}

	pub fn get_host_and_port(&self) -> String {
		format!(
			"{}{}",
			self.uri.host().unwrap(),
			if let Some(port) = self.uri.port_u16() {
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
		Some(data[(charset_index + 8)..data.find(';').unwrap_or_else(|| data.len())].to_string())
	}

	pub fn get_protocol(&self) -> String {
		// TODO support X-Forwarded-Proto
		self.uri.scheme_str().unwrap_or("http").to_string()
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
		format!("{}.{}", self.version.0, self.version.1)
	}

	pub fn get_version_major(&self) -> u8 {
		self.version.0
	}

	pub fn get_version_minor(&self) -> u8 {
		self.version.1
	}

	pub fn get_header(&self, field: &str) -> Option<String> {
		self.headers.iter().find_map(|(key, value)| {
			if key.to_lowercase() == field.to_lowercase() {
				Some(value.join("\n"))
			} else {
				None
			}
		})
	}
	pub fn get_headers(&self) -> &HashMap<String, Vec<String>> {
		&self.headers
	}
	pub fn set_header(&mut self, field: &str, value: &str) {
		self.headers
			.insert(field.to_string(), vec![value.to_string()]);
	}
	pub fn append_header(&mut self, key: String, value: String) {
		if let Some(headers) = self.headers.get_mut(&key) {
			headers.push(value);
		} else {
			self.headers.insert(key, vec![value]);
		}
	}
	pub fn remove_header(&mut self, field: &str) {
		self.headers.remove(field);
	}

	pub fn get_query(&self) -> &HashMap<String, String> {
		&self.query
	}

	pub fn get_params(&self) -> &HashMap<String, String> {
		&self.params
	}

	pub fn get_cookies(&self) -> &Vec<Cookie> {
		&self.cookies
	}

	pub fn get_cookie(&self, name: &str) -> Option<&Cookie> {
		self.cookies.iter().find(|cookie| cookie.key == name)
	}

	pub fn get_hyper_request(&self) -> &HyperRequest {
		&self.hyper_request
	}

	pub fn get_hyper_request_mut(&mut self) -> &mut HyperRequest {
		&mut self.hyper_request
	}

	pub fn set_max_body_size(&mut self, max_size: usize) {
		self.max_request_body = max_size;
	}

	pub fn get_max_body_size(&self) -> usize {
		self.max_request_body
	}
}

#[cfg(debug_assertions)]
impl Debug for Request {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Request")
			.field("socket_addr", &self.socket_addr)
			.field("body", &self.body)
			.field("method", &self.method)
			.field("uri", &self.uri)
			.field("version", &self.version)
			.field("headers", &self.headers)
			.field("query", &self.query)
			.field("params", &self.params)
			.field("cookies", &self.cookies)
			.finish()
	}
}

#[cfg(not(debug_assertions))]
impl Debug for Request {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "[Request {} {}]", self.method, self.get_path())
	}
}
