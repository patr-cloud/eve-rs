use crate::context::Context;
use hyper::Error;
use regex::Regex;
use std::{future::Future, marker::PhantomData, pin::Pin};

pub type NextHandler<TContext> = Box<
	dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<TContext, Error>> + Send>> + Send + Sync,
>;

#[async_trait]
pub trait Middleware<TContext: Context + Send + Sync> {
	async fn run(&self, context: TContext, next: NextHandler<TContext>) -> Result<TContext, Error>;
}

#[derive(Clone)]
pub(crate) struct MiddlewareHandler<TContext, TMiddleware>
where
	TContext: Context + Clone + Send + Sync,
	TMiddleware: Middleware<TContext> + Clone + Send + Sync,
{
	pub(crate) path_match: Regex,
	pub(crate) handler: TMiddleware,
	phantom: PhantomData<TContext>,
}

impl<TContext, TMiddleware> MiddlewareHandler<TContext, TMiddleware>
where
	TContext: Context + Clone + Send + Sync,
	TMiddleware: Middleware<TContext> + Clone + Send + Sync,
{
	pub(crate) fn new(path: &str, handler: TMiddleware, is_endpoint: bool) -> Self {
		let path = path
			.replace('.', "\\.") // Specifically, match the dot. This ain't a regex character
			.replace('*', "([^\\/].)+") // Match anything that's not a /, but at least 1 character
			.replace("**", "(.)+"); //Match anything

		// Make a variable out of anything that begins with a : and has a-z, A-Z, 0-9, '_'
		let mut path = Regex::new(":(?P<var>([a-zA-Z0-9_]+))")
			.unwrap()
			// Match that variable with anything that has a-z, A-Z, 0-9, '_', '.' and a '-'
			.replace_all(&path, "(?P<$var>([a-zA-Z0-9_\\.-]+))")
			.to_string();

		// Make sure it always begins with a /
		if path.starts_with("./") {
			path = path[1..].to_string();
		} else if !path.starts_with('/') {
			path = format!("/{}", path);
		}

		// if there's a trailing /, remove it
		if path.ends_with('/') {
			path = path[..(path.len() - 1)].to_owned();
		}

		if path.is_empty() {
			// If there's nothing left, set the middleware to /
			path.push('/');
		} else {
			// If there's something to match with,
			// add the Regex to mention that both / and non / should match at the end of the url
			path.push_str("[/]?");
		}

		// If this is only supposed to match an endpoint URL, make sure the Regex only allows the end of the path
		if is_endpoint {
			path.push('$');
		}

		MiddlewareHandler {
			path_match: Regex::new(&path).unwrap(),
			handler,
			phantom: PhantomData,
		}
	}

	pub(crate) fn is_match(&self, url: &str) -> bool {
		self.path_match.is_match(url)
	}
}
