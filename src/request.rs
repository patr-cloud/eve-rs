use crate::{cookie::Cookie, HttpMethod};
use serde::Deserialize;
use serde_json::Error;
use std::{
	collections::HashMap,
	fmt::{Debug, Formatter},
	str::{self, Utf8Error},
};

pub struct Request {
	pub(crate) body: Vec<u8>,
	pub(crate) method: HttpMethod,
	pub(crate) path: String,
	pub(crate) version: (u8, u8),
	pub(crate) headers: HashMap<String, Vec<String>>,
	pub(crate) query: HashMap<String, String>,
	pub(crate) params: HashMap<String, String>,
	pub(crate) cookies: Vec<Cookie>,
}

impl Request {
	pub fn get_body_bytes(&self) -> &[u8] {
		&self.body
	}

	pub fn get_body(&self) -> Result<&str, Utf8Error> {
		str::from_utf8(&self.body)
	}

	pub fn get_body_as<'a, T>(&self, body: &'a str) -> Result<T, Error>
	where
		T: Deserialize<'a>,
	{
		serde_json::from_str(body)
	}

	pub fn get_method(&self) -> &HttpMethod {
		&self.method
	}

	pub fn get_path(&self) -> &str {
		&self.path
	}

	pub fn get_version(&self) -> &str {
		todo!()
		//&format!("{}.{}", self.version.0, self.version.1)
	}

	pub fn get_version_major(&self) -> u8 {
		self.version.0
	}

	pub fn get_version_minor(&self) -> u8 {
		self.version.1
	}

	pub fn get_headers(&self) -> &HashMap<String, Vec<String>> {
		&self.headers
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
}

impl Debug for Request {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"<HTTP Request {} {}>",
			self.get_method(),
			self.get_path()
		)
	}
}
