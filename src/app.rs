use crate::{
	context::Context, http_method::HttpMethod, middleware::Middleware,
	middleware_handler::MiddlewareHandler,
};

use std::{future::Future, pin::Pin, sync::Arc};

use hyper::Error;

fn chained_run<TContext, TMiddleware>(
	mut context: TContext,
	nodes: Arc<Vec<MiddlewareHandler<TContext, TMiddleware>>>,
	i: usize,
) -> Pin<Box<dyn Future<Output = Result<TContext, Error>> + Send>>
where
	TContext: 'static + Context + Clone + Send + Sync,
	TMiddleware: 'static + Middleware<TContext> + Clone + Send + Sync,
{
	Box::pin(async move {
		if let Some(m) = nodes.clone().get(i) {
			m.handler
				.run(
					context,
					Box::new(move |context| chained_run(context, nodes.clone(), i + 1)),
				)
				.await
		} else {
			let method = context.get_method().to_string();
			let path = context.get_path().to_string();
			context
				.status(404)
				.body(&format!("Cannot {} route {}", method, path));
			Ok(context)
		}
	})
}

pub struct App<TContext, TMiddleware>
where
	TContext: 'static + Context + Clone + Send + Sync,
	TMiddleware: 'static + Middleware<TContext> + Clone + Send + Sync,
{
	get_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	post_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	put_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	delete_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	head_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	options_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	connect_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	patch_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
	trace_stack: Vec<MiddlewareHandler<TContext, TMiddleware>>,
}

impl<TContext, TMiddleware> App<TContext, TMiddleware>
where
	TContext: 'static + Context + Clone + Send + Sync,
	TMiddleware: 'static + Middleware<TContext> + Clone + Send + Sync,
{
	pub fn new() -> Self {
		App {
			get_stack: vec![],
			post_stack: vec![],
			put_stack: vec![],
			delete_stack: vec![],
			head_stack: vec![],
			options_stack: vec![],
			connect_stack: vec![],
			patch_stack: vec![],
			trace_stack: vec![],
		}
	}

	pub fn get(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.get_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn post(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.post_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn put(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.put_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn delete(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.delete_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn head(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.head_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn options(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.options_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn connect(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.connect_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn patch(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.patch_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn trace(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.trace_stack
				.push(self.create_middleware_handler(path, handler.clone(), true));
		});
	}

	pub fn use_middleware(&mut self, path: &str, middlewares: &[TMiddleware]) {
		middlewares.into_iter().for_each(|handler| {
			self.get_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.post_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.put_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.delete_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.head_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.options_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.connect_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.patch_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
			self.trace_stack
				.push(self.create_middleware_handler(path, handler.clone(), false));
		});
	}

	pub fn use_sub_app(&mut self, base_path: &str, sub_app: App<TContext, TMiddleware>) {
		let base_path = {
			if base_path == "/" {
				"".to_string()
			} else {
				let mut formatted_base_path = base_path.to_string();

				// If it ends with /, remove it
				if base_path.ends_with('/') {
					formatted_base_path = base_path[..(base_path.len() - 1)].to_string();
				}

				// If it doesn't begin with a /, add it
				if !base_path.starts_with('/') {
					formatted_base_path = format!("/{}", base_path);
				}

				formatted_base_path
			}
		};

		let App {
			get_stack,
			post_stack,
			put_stack,
			delete_stack,
			head_stack,
			options_stack,
			connect_stack,
			patch_stack,
			trace_stack,
		} = sub_app;

		for handler in get_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in post_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in put_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in delete_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in head_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in options_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in connect_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in patch_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
		for handler in trace_stack {
			self.get_stack.push(self.create_middleware_handler(
				&format!("{}{}", base_path, handler.mounted_url),
				handler.handler,
				handler.is_endpoint,
			));
		}
	}

	pub async fn resolve(&self, context: TContext) -> Result<TContext, hyper::Error> {
		let stack = self.get_middleware_stack(context.get_method(), context.get_path());
		chained_run(context, Arc::new(stack), 0).await
	}

	fn create_middleware_handler(
		&self,
		path: &str,
		handler: TMiddleware,
		is_endpoint: bool,
	) -> MiddlewareHandler<TContext, TMiddleware> {
		MiddlewareHandler::new(path, handler, is_endpoint)
	}

	fn get_middleware_stack(
		&self,
		method: &HttpMethod,
		path: &str,
	) -> Vec<MiddlewareHandler<TContext, TMiddleware>> {
		let mut stack = vec![];
		let route_stack = match method {
			HttpMethod::Get => &self.get_stack,
			HttpMethod::Post => &self.post_stack,
			HttpMethod::Put => &self.put_stack,
			HttpMethod::Delete => &self.delete_stack,
			HttpMethod::Head => &self.head_stack,
			HttpMethod::Options => &self.options_stack,
			HttpMethod::Connect => &self.connect_stack,
			HttpMethod::Patch => &self.patch_stack,
			HttpMethod::Trace => &self.trace_stack,
			_ => unreachable!("Getting a middleware stack for use? What?"),
		};
		for handler in route_stack {
			if handler.is_match(path) {
				stack.push(handler.clone());
			}
		}
		stack
	}
}
