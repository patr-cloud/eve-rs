use serde_json::Value;
use std::{collections::HashMap, str};

pub trait Context {

	// Stuff for request
	fn get_raw_body(&self) -> &[u8];
	fn get_body_string(&self) -> Result<&str, str::Utf8Error> {
		str::from_utf8(self.get_raw_body())
	}
	fn get_body_object(&self) -> Result<Value, serde_json::Error> {
		serde_json::from_slice(self.get_raw_body())
	}
	fn get_cookies(&self) -> Vec<Cookie>;
	fn get_host(&self) -> &str;
	fn get_ip(&self) -> &str;
	fn get_ips(&self) -> Vec<&str>;
	fn get_method(&self) -> &str;
	fn get_path(&self) -> &str;
	fn get_params(&self) -> HashMap<String, String>;
	fn get_protocol(&self) -> &str;
	fn get_query(&self) -> HashMap<String, Value>;
	fn is_secure(&self) -> bool {
		self.get_protocol() == "https"
	}
	fn get(&self, header: &str) -> Option<&str>;
}

pub struct Cookie {
	pub key: String,
	pub value: String,
	pub options: CookieOptions,
}

pub struct CookieOptions {
	pub domain: String,
	pub path: String,
	pub expires: u64,
	pub http_only: bool,
	pub max_age: u64,
	pub secure: bool,
	pub signed: bool,
	pub same_site: Option<SameSite>,
}

#[derive(PartialEq)]
pub enum SameSite {
	Strict,
	Lax,
}

pub struct DefaultContext {}
