#[derive(Clone, Debug)]
pub struct Cookie {
	pub key: String,
	pub value: String,
	pub options: CookieOptions,
}

impl Cookie {
	pub fn to_header_string(&self) -> String {
		let name = &self.key;
		let value = &self.value;
		let options = &self.options;
		
		let mut pieces = vec![format!("Path={}", self.options.path)];

		if options.expires > 0 {
			pieces.push(format!("Expires={}", options.expires));
		}

		if options.max_age > 0 {
			pieces.push(format!("Max-Age={}", options.max_age));
		}

		if !options.domain.is_empty() {
			pieces.push(format!("Domain={}", options.domain));
		}

		if options.secure {
			pieces.push("Secure".to_owned());
		}

		if options.http_only {
			pieces.push("HttpOnly".to_owned());
		}

		if let Some(ref same_site) = options.same_site {
			match same_site {
				SameSite::Strict => pieces.push("SameSite=Strict".to_owned()),
				SameSite::Lax => pieces.push("SameSite=Lax".to_owned()),
			};
		}

		format!("{}={}; {}", name, value, pieces.join(", "))
	}
}

#[derive(Default, Clone, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
pub enum SameSite {
	Strict,
	Lax,
}
