use std::{
	convert::TryFrom,
	fmt::{Display, Error, Formatter},
	str::FromStr,
};

use hyper::Method;

use crate::error::EveError;

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
			}
		)
	}
}

impl FromStr for HttpMethod {
	type Err = EveError;

	fn from_str(method: &str) -> Result<Self, Self::Err> {
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
			_ => Err(EveError::UnknownHttpMethod(method.to_string())),
		}
	}
}

impl TryFrom<&Method> for HttpMethod {
	type Error = EveError;

	fn try_from(method: &Method) -> Result<Self, Self::Error> {
		match method {
			&Method::GET => Ok(Self::Get),
			&Method::POST => Ok(Self::Post),
			&Method::PUT => Ok(Self::Put),
			&Method::DELETE => Ok(Self::Delete),
			&Method::HEAD => Ok(Self::Head),
			&Method::OPTIONS => Ok(Self::Options),
			&Method::CONNECT => Ok(Self::Connect),
			&Method::PATCH => Ok(Self::Patch),
			&Method::TRACE => Ok(Self::Trace),
			method => {
				log::warn!(
					"Unable to parse {} as an http method. Ignoring...",
					method
				);
				Err(EveError::UnknownHttpMethod(method.to_string()))
			}
		}
	}
}
