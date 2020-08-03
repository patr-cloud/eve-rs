use std::{
	collections::HashMap,
	fmt::{Debug, Formatter, Result as FmtResult},
};

#[derive(Clone)]
pub struct Response {
	pub(crate) body: Vec<u8>,
	pub(crate) status: u16,
	pub(crate) headers: HashMap<String, Vec<String>>,
}

impl Response {
	pub fn new() -> Self {
		Response {
			body: vec![],
			status: 200,
			headers: HashMap::new(),
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
	pub fn set_status(&mut self, code: u16) {
		self.status = code;
	}

	pub fn set_content_length(&mut self, length: u128) {
		self.set_header("Content-Length", &format!("{}", length));
	}

	pub fn get_headers(&self) -> &HashMap<String, Vec<String>> {
		&self.headers
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
	pub fn set_header(&mut self, key: &str, value: &str) {
		self.headers
			.insert(key.to_string(), vec![value.to_string()]);
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

	pub fn get_content_type(&self) -> String {
		let c_type = self
			.get_header("Content-Type")
			.unwrap_or_else(|| "text/plain".to_string());
		c_type.split(';').next().unwrap_or("").to_string()
	}
	pub fn set_content_type(&mut self, content_type: &str) {
		self.set_header("Content-Type", content_type);
	}

	pub fn redirect(&mut self, url: &str) {
		if self.status == 200 {
			self.set_status(302);
		}
		self.set_header("Location", url);
	}

	pub fn attachment(&mut self, file_name: Option<&str>) {
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
		);
	}

	pub fn get_last_modified(&self) -> Option<String> {
		self.get_header("Last-Modified")
	}
	pub fn set_last_modified(&mut self, last_modified: &str) {
		self.set_header("Last-Modified", last_modified);
	}

	pub fn set_etag(&mut self, etag: &str) {
		self.set_header("ETag", etag);
	}

	pub fn get_body(&self) -> &Vec<u8> {
		&self.body
	}
	pub fn set_body(&mut self, data: &str) {
		self.body = data.as_bytes().to_vec();
	}
	pub fn set_body_bytes(&mut self, data: &[u8]) {
		self.body = data.to_vec();
	}
}

impl Debug for Response {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if cfg!(debug_assertions) {
			f.debug_struct("Request")
				.field("body", &self.body)
				.field("status", &self.status)
				.field("headers", &self.headers)
				.finish()
		} else {
			write!(f, "[Response {}]", self.status)
		}
	}
}

impl Default for Response {
	fn default() -> Self {
		Response {
			body: vec![],
			status: 200,
			headers: HashMap::new(),
		}
	}
}
