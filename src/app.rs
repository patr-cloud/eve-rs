use crate::{
	context::Context,
	http_method::HttpMethod,
	middleware::{Middleware, MiddlewareHandler},
	routeable::Routeable,
};

use std::collections::HashMap;

pub struct App<TContext, TMiddleware>
where
	TContext: Context + Clone,
	TMiddleware: Middleware<TContext> + Clone,
{
	route_stack: HashMap<HttpMethod, Vec<MiddlewareHandler<TContext, TMiddleware>>>,
}

impl<
		TContext: 'static + Context + Send + Clone,
		TMiddleware: 'static + Middleware<TContext> + Clone + Send,
	> App<TContext, TMiddleware>
{
	pub fn new<ContextType: Context, MiddlewareType: Middleware<ContextType>>() -> Self {
		App {
			route_stack: HashMap::new(),
		}
	}

	fn add_to_stack(&mut self, method: &HttpMethod, path: &str, middleware: TMiddleware) {
		if let Some(stack) = self.route_stack.get_mut(&method) {
			stack.push(MiddlewareHandler::new(path.to_string(), middleware));
		} else {
			self.route_stack.insert(
				method.clone(),
				vec![MiddlewareHandler::new(path.to_string(), middleware)],
			);
		}
	}

	fn get_middleware_stack(
		&self,
		method: &HttpMethod,
		path: &str,
	) -> Vec<MiddlewareHandler<TContext, TMiddleware>> {
		let mut stack = vec![];
		for handler in self.route_stack.get(method).unwrap_or(&Vec::default()) {
			if handler.is_match(path) {
				stack.push(handler.clone());
			}
		}
		stack
	}
}

impl<
		TContext: 'static + Context + Clone + Send,
		TMiddleware: 'static + Middleware<TContext> + Clone + Send,
	> Routeable<TContext, TMiddleware> for App<TContext, TMiddleware>
{
	fn use_middleware(&mut self, path: &str, middleware: TMiddleware) {
		for method in &[
			HttpMethod::Get,
			HttpMethod::Post,
			HttpMethod::Put,
			HttpMethod::Delete,
			HttpMethod::Head,
			HttpMethod::Options,
			HttpMethod::Connect,
			HttpMethod::Patch,
			HttpMethod::Trace,
			HttpMethod::Use,
		] {
			self.add_to_stack(method, path, middleware.clone());
		}
	}

	fn get(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Get, path, middleware);
	}
	fn post(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Post, path, middleware);
	}
	fn put(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Put, path, middleware);
	}
	fn delete(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Delete, path, middleware);
	}
	fn head(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Head, path, middleware);
	}
	fn options(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Options, path, middleware);
	}
	fn connect(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Connect, path, middleware);
	}
	fn patch(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Patch, path, middleware);
	}
	fn trace(&mut self, path: &str, middleware: TMiddleware) {
		self.add_to_stack(&HttpMethod::Trace, path, middleware);
	}
}
