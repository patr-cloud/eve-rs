use hyper::Method;
use std::{
	fmt::{Display, Error, Formatter},
	str::FromStr,
};

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
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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

impl FromStr for HttpMethod {
	type Err = String;

	fn from_str(method: &str) -> Result<Self, String> {
		match method.to_lowercase().as_ref() {
			"get" => Ok(HttpMethod::Get),
			"post" => Ok(HttpMethod::Post),
			"put" => Ok(HttpMethod::Put),
			"delete" => Ok(HttpMethod::Delete),
			"head" => Ok(HttpMethod::Head),
			"options" => Ok(HttpMethod::Options),
			"connect" => Ok(HttpMethod::Connect),
			"patch" => Ok(HttpMethod::Patch),
			"trace" => Ok(HttpMethod::Trace),
			_ => Err(format!(
				"Could not parse a suitable HTTP Method for string: '{}'",
				method
			)),
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
