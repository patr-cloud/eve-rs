use std::{fmt::Debug, marker::PhantomData};

use regex::Regex;

use crate::{Context, Error, Middleware};

pub(crate) struct MiddlewareHandler<TContext, TMiddleware, TError>
where
	TContext: Context + Debug + Send + Sync,
	TMiddleware: Middleware<TContext, TError> + Clone + Send + Sync,
	TError: Error + Send + Sync,
{
	pub(crate) is_endpoint: bool,
	pub(crate) mounted_url: String,
	pub(crate) path_match: Regex,
	pub(crate) handler: TMiddleware,
	phantom_context: PhantomData<TContext>,
	phantom_error: PhantomData<TError>,
}

impl<TContext, TMiddleware, TError> Clone
	for MiddlewareHandler<TContext, TMiddleware, TError>
where
	TContext: Context + Debug + Send + Sync,
	TMiddleware: Middleware<TContext, TError> + Clone + Send + Sync,
	TError: Error + Send + Sync,
{
	fn clone(&self) -> Self {
		MiddlewareHandler {
			is_endpoint: self.is_endpoint,
			mounted_url: self.mounted_url.clone(),
			path_match: self.path_match.clone(),
			handler: self.handler.clone(),
			phantom_context: PhantomData,
			phantom_error: PhantomData,
		}
	}
}

impl<TContext, TMiddleware, TError>
	MiddlewareHandler<TContext, TMiddleware, TError>
where
	TContext: Context + Debug + Send + Sync,
	TMiddleware: Middleware<TContext, TError> + Clone + Send + Sync,
	TError: Error + Send + Sync,
{
	pub(crate) fn new(
		path: &str,
		handler: TMiddleware,
		is_endpoint: bool,
	) -> Self {
		let mut mounted_url = path.to_string();

		// Make sure it always begins with a /
		if mounted_url.starts_with("./") {
			mounted_url = mounted_url[1..].to_string();
		} else if !path.starts_with('/') {
			mounted_url = format!("/{}", mounted_url);
		}

		// if there's a trailing /, remove it
		if mounted_url.ends_with('/') {
			mounted_url = path[..(path.len() - 1)].to_string();
		}

		// If there's nothing left, set the middleware to /
		if mounted_url.is_empty() {
			mounted_url.push('/');
		}

		let mut regex_path = mounted_url
			.replace('\\', "\\\\")
			.replace('[', "\\[")
			.replace(']', "\\]")
			.replace('?', "\\?")
			.replace('+', "\\+")
			.replace('{', "\\{")
			.replace('}', "\\}")
			.replace('(', "\\)")
			.replace('(', "\\)")
			.replace('|', "\\|")
			.replace('^', "\\^")
			.replace('$', "\\$")
			.replace('.', "\\.") // Specifically, match the dot. This ain't a regex character
			.replace("**", "(.+)") // Match anything [ NOTE: first replace `**` and then replace remaining `*` ]
			.replace('*', "([^/]+)"); // Match anything that's not a /, but at least 1 character

		// Make a variable out of anything that begins with a : and has a-z,
		// A-Z, 0-9, '_'
		regex_path = Regex::new(":(?P<var>([a-zA-Z0-9_]+))")
			.unwrap()
			// Match that variable with anything that isn't a `/`
			.replace_all(&regex_path, "(?P<$var>([^\\s/]+))")
			.to_string();

		if regex_path != "/" {
			// If there's something to match with,
			// add the Regex to mention that both / and non / should match at
			// the end of the url
			regex_path.push_str("[/]?");
		}

		// If this is only supposed to match an endpoint URL, make sure the
		// Regex only allows the end of the path
		if is_endpoint {
			regex_path.push('$');
		}

		MiddlewareHandler {
			is_endpoint,
			mounted_url,
			path_match: Regex::new(&regex_path).unwrap(),
			handler,
			phantom_context: PhantomData,
			phantom_error: PhantomData,
		}
	}

	pub(crate) fn is_match(&self, url: &str) -> bool {
		self.path_match.is_match(url)
	}
}
