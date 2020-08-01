use crate::{cookie::Cookie, request::Request, response::Response, HttpMethod};

use serde_json::Value;
use std::str::{self, Utf8Error};

pub trait Context {
	fn create(request: Request) -> Self;
	fn get_request(&self) -> &Request;
	fn get_request_mut(&mut self) -> &mut Request;
	fn get_response(&self) -> &Response;
	fn get_response_mut(&mut self) -> &mut Response;

	fn get_body(&self) -> Result<&str, Utf8Error> {
		self.get_request().get_body()
	}
	fn json(&mut self, body: Value) -> &mut Self {
		self.content_type("application/json")
			.body(&body.to_string())
	}
	fn body(&mut self, string: &str) -> &mut Self {
		self.get_response_mut().set_body(string);
		self
	}
	fn body_bytes(&mut self, bytes: &[u8]) -> &mut Self {
		self.get_response_mut().set_body_bytes(bytes);
		self
	}

	fn get_method(&self) -> &HttpMethod {
		self.get_request().get_method()
	}

	fn status(&mut self, code: u16) -> &mut Self {
		self.get_response_mut().set_status(code);
		self
	}

	fn content_type(&mut self, c_type: &str) -> &mut Self {
		self.header("Content-Type", c_type)
	}

	fn redirect(&mut self, destination: &str) -> &mut Self {
		self.status(302).header("Location", destination)
	}

	fn get_path(&self) -> &str {
		self.get_request().get_path()
	}

	fn get_header(&self, key: &str) -> Option<&Vec<String>> {
		self.get_request().get_headers().get(key)
	}
	fn header(&mut self, key: &str, value: &str) -> &mut Self {
		self.get_response_mut()
			.set_header(key.to_owned(), value.to_owned());
		self
	}

	fn cookie(&mut self, cookie: &Cookie) -> &mut Self {
		self.header("Set-Cookie", &cookie.to_header_string())
	}
}
