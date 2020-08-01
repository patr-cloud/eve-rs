use crate::{cookie::Cookie, CookieOptions, HttpMethod, SameSite};
use hyper::{body, Body, Request as HyperRequest, Version};
use serde::Deserialize;
use serde_json::Error;
use std::{
	collections::HashMap,
	fmt::Debug,
	str::{self, Utf8Error},
};

#[derive(Clone, Debug)]
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
	pub async fn from_hyper(req: HyperRequest<Body>) -> Self {
		let (parts, body) = req.into_parts();
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
			body: body::to_bytes(body).await.unwrap().to_vec(),
			method: HttpMethod::from(parts.method),
			path: parts.uri.path().to_string(),
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
				query
					.split('&')
					.filter_map(|kv| {
						if !kv.contains('=') {
							None
						} else {
							let mut items = kv.split('=').map(String::from);
							let key = if let Some(key) = items.next() {
								key
							} else {
								return None;
							};
							let value = if let Some(value) = items.next() {
								value
							} else {
								return None;
							};
							Some((key, value))
						}
					})
					.collect::<HashMap<String, String>>()
			} else {
				HashMap::new()
			},
			params: HashMap::new(),
			cookies: if let Some(header) = headers.remove("Cookie") {
				header
					.into_iter()
					.map(|header| {
						let mut options = CookieOptions::default();

						let mut pieces = header.split(';');

						let mut key_pair = pieces.next().unwrap().split('=');

						let key = key_pair.next().unwrap_or("").to_owned();
						let value = key_pair.next().unwrap_or("").to_owned();

						for option in pieces {
							let mut option_key_pair = option.split('=');

							if let Some(option_key) = option_key_pair.next() {
								match option_key.to_lowercase().trim() {
									"expires" => {
										options.expires = option_key_pair
											.next()
											.unwrap_or("0")
											.parse::<u64>()
											.unwrap_or(0)
									}
									"max-age" => {
										options.max_age = option_key_pair
											.next()
											.unwrap_or("0")
											.parse::<u64>()
											.unwrap_or(0)
									}
									"domain" => {
										options.domain =
											option_key_pair.next().unwrap_or("").to_owned()
									}
									"path" => {
										options.path =
											option_key_pair.next().unwrap_or("").to_owned()
									}
									"secure" => options.secure = true,
									"httponly" => options.http_only = true,
									"samesite" => {
										if let Some(same_site_value) = option_key_pair.next() {
											match same_site_value.to_lowercase().as_ref() {
												"strict" => {
													options.same_site = Some(SameSite::Strict)
												}
												"lax" => options.same_site = Some(SameSite::Lax),
												_ => (),
											};
										}
									}
									_ => (),
								};
							}
						}

						Cookie {
							key,
							value,
							options,
						}
					})
					.collect::<Vec<Cookie>>()
			} else {
				vec![]
			},
		}
	}

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

	pub fn get_version(&self) -> String {
		format!("{}.{}", self.version.0, self.version.1)
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
