use hyper::Method;
use std::fmt::{Display, Formatter, Result};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum HttpMethod {
	Get,
	Post,
	Put,
	Delete,
	Head,
	Options,
	Connect,
	Patch,
	Trace,
	Use,
}

impl Display for HttpMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{}",
			match self {
				HttpMethod::Get => "GET",
				HttpMethod::Head => "HEAD",
				HttpMethod::Options => "OPTIONS",
				HttpMethod::Connect => "CONNECT",
				HttpMethod::Delete => "DELETE",
				HttpMethod::Patch => "PATCH",
				HttpMethod::Post => "POST",
				HttpMethod::Put => "PUT",
				HttpMethod::Trace => "TRACE",
				HttpMethod::Use => "UNKNOWN",
			}
		)
	}
}

impl HttpMethod {
	pub fn from_str(method: &str) -> Option<Self> {
		match method.to_lowercase().as_ref() {
			"get" => Some(HttpMethod::Get),
			"post" => Some(HttpMethod::Post),
			"put" => Some(HttpMethod::Put),
			"delete" => Some(HttpMethod::Delete),
			"head" => Some(HttpMethod::Head),
			"options" => Some(HttpMethod::Options),
			"connect" => Some(HttpMethod::Connect),
			"patch" => Some(HttpMethod::Patch),
			"trace" => Some(HttpMethod::Trace),
			_ => None,
		}
	}
}

impl From<Method> for HttpMethod {
	fn from(method: Method) -> Self {
		match method {
			Method::GET => HttpMethod::Get,
			Method::POST => HttpMethod::Post,
			Method::PUT => HttpMethod::Put,
			Method::DELETE => HttpMethod::Delete,
			Method::HEAD => HttpMethod::Head,
			Method::OPTIONS => HttpMethod::Options,
			Method::CONNECT => HttpMethod::Connect,
			Method::PATCH => HttpMethod::Patch,
			Method::TRACE => HttpMethod::Trace,
			_ => HttpMethod::Use,
		}
	}
}
