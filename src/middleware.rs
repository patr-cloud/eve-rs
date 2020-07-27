use crate::context::Context;
use hyper::Error;
use regex::Regex;
use std::{future::Future, marker::PhantomData, pin::Pin};

pub type NextHandler<TContext> = Box<
//	dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<TContext, Error>> + Send>> + Send + Sync,
	dyn Fn(TContext) -> Result<TContext, Error>,
>;

//#[async_trait]
pub trait Middleware<TContext: Context + Send + Sync> {
	fn run(&self, context: TContext, next: NextHandler<TContext>) -> Result<TContext, Error>;
}

#[derive(Clone)]
pub struct MiddlewareHandler<
	TContext: Context + Clone + Send + Sync,
	TMiddleware: Middleware<TContext> + Clone + Send + Sync,
> {
	pub(crate) path_match: Regex,
	pub(crate) handler: TMiddleware,
	phantom_data: PhantomData<TContext>,
}

impl<
		TContext: Context + Clone + Send + Sync,
		TMiddleware: Middleware<TContext> + Clone + Send + Sync,
	> MiddlewareHandler<TContext, TMiddleware>
{
	pub(crate) fn new(path: String, middleware: TMiddleware) -> Self {
		let path = path
			.replace('.', "\\.")
			.replace('*', "(^\\\\.)+")
			.replace("**", "(.)+");

		let path = Regex::new(":(?P<var>([a-zA-Z0-9_]+))")
			.unwrap()
			.replace_all(&path, "(?P<$var>([a-zA-Z0-9_\\.-]+))");

		MiddlewareHandler {
			path_match: Regex::new(&path).unwrap(),
			handler: middleware,
			phantom_data: PhantomData::default(),
		}
	}

	pub(crate) fn is_match(&self, url: &str) -> bool {
		self.path_match.is_match(url)
	}
}
