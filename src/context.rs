use std::{
	net::IpAddr,
	str::{self, Utf8Error},
};

use serde_json::Value;

use crate::{cookie::Cookie, request::Request, response::Response, HttpMethod};

pub trait Context {
	fn get_request(&self) -> &Request;
	fn get_request_mut(&mut self) -> &mut Request;
	fn get_response(&self) -> &Response;
	fn take_response(self) -> Response;
	fn get_response_mut(&mut self) -> &mut Response;

	fn get_body(&self) -> Result<String, Utf8Error> {
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

	fn get_status(&self) -> u16 {
		self.get_response().get_status()
	}
	fn get_status_message(&self) -> &str {
		self.get_response().get_status_message()
	}
	fn status(&mut self, code: u16) -> &mut Self {
		self.get_response_mut().set_status(code);
		self
	}

	fn content_type(&mut self, content_type: &str) -> &mut Self {
		self.get_response_mut().set_content_type(content_type);
		self
	}

	fn content_length(&mut self, length: usize) -> &mut Self {
		self.get_response_mut().set_content_length(length);
		self
	}

	fn redirect(&mut self, destination: &str) -> &mut Self {
		self.get_response_mut().redirect(destination);
		self
	}

	fn attachment(&mut self, file_name: Option<&str>) -> &mut Self {
		self.get_response_mut().attachment(file_name);
		self
	}

	fn get_path(&self) -> String {
		self.get_request().get_path()
	}

	fn get_full_url(&self) -> String {
		self.get_request().get_full_url()
	}

	fn get_origin(&self) -> Option<String> {
		self.get_request().get_origin()
	}

	fn get_query_string(&self) -> String {
		self.get_request().get_query_string()
	}

	fn get_host(&self) -> String {
		self.get_request().get_host()
	}

	fn get_host_and_port(&self) -> String {
		self.get_request().get_host_and_port()
	}

	fn get_content_type(&self) -> String {
		self.get_request().get_content_type()
	}

	fn get_charset(&self) -> Option<String> {
		self.get_request().get_charset()
	}

	fn get_protocol(&self) -> String {
		self.get_request().get_protocol()
	}

	fn is_secure(&self) -> bool {
		self.get_request().is_secure()
	}

	fn get_ip(&self) -> IpAddr {
		self.get_request().get_ip()
	}

	fn is(&self, mimes: &[&str]) -> bool {
		self.get_request().is(mimes)
	}

	// TODO content negotiation
	// See: https://koajs.com/#request content negotiation

	fn get_header(&self, key: &str) -> Option<String> {
		self.get_request().get_header(key)
	}
	fn header(&mut self, key: &str, value: &str) -> &mut Self {
		self.get_response_mut().set_header(key, value);
		self
	}
	fn append_header(&mut self, key: &str, value: &str) -> &mut Self {
		self.get_response_mut().append_header(key, value);
		self
	}
	fn remove_header(&mut self, key: &str) -> &mut Self {
		self.get_response_mut().remove_header(key);
		self
	}

	fn get_cookie(&self, name: &str) -> Option<&Cookie> {
		self.get_request().get_cookie(name)
	}
	fn get_cookies(&self) -> &Vec<Cookie> {
		self.get_request().get_cookies()
	}
	fn cookie(&mut self, cookie: Cookie) -> &mut Self {
		self.get_response_mut().set_cookie(cookie);
		self
	}

	fn last_modified(&mut self, last_modified: &str) -> &mut Self {
		self.get_response_mut().set_last_modified(last_modified);
		self
	}

	fn etag(&mut self, etag: &str) -> &mut Self {
		self.get_response_mut().set_etag(etag);
		self
	}
}

#[derive(Debug)]
pub struct DefaultContext {
	request: Request,
	response: Response,
	body: Option<Value>,
}

impl DefaultContext {
	pub fn get_body_object(&self) -> Option<&Value> {
		self.body.as_ref()
	}

	pub fn set_body_object(&mut self, body: Value) {
		self.body = Some(body);
	}

	pub fn new(request: Request) -> Self {
		DefaultContext {
			request,
			response: Default::default(),
			body: None,
		}
	}
}

impl Context for DefaultContext {
	fn get_request(&self) -> &Request {
		&self.request
	}

	fn get_request_mut(&mut self) -> &mut Request {
		&mut self.request
	}

	fn get_response(&self) -> &Response {
		&self.response
	}

	fn take_response(self) -> Response {
		self.response
	}

	fn get_response_mut(&mut self) -> &mut Response {
		&mut self.response
	}
}

pub fn default_context_generator<TState>(
	request: Request,
	_: &TState,
) -> DefaultContext {
	DefaultContext::new(request)
}
