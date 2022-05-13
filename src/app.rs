use std::{
	collections::HashMap,
	fmt::Debug,
	future::Future,
	pin::Pin,
	sync::Arc,
};

use crate::{
	context::Context,
	error::Error,
	http_method::HttpMethod,
	middleware::Middleware,
	middleware_handler::MiddlewareHandler,
	DefaultContext,
	DefaultError,
	DefaultMiddleware,
	Request,
	Response,
};

type ContextGeneratorFn<TContext = DefaultContext, TState = ()> =
	fn(Request, Response, &TState) -> TContext;
type ErrorHandlerFn<TError> = fn(Response, TError);

fn chained_run<TContext, TMiddleware, TError>(
	mut context: TContext,
	nodes: Arc<Vec<MiddlewareHandler<TContext, TMiddleware, TError>>>,
	i: usize,
) -> Pin<Box<dyn Future<Output = Result<TContext, TError>> + Send>>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext, TError> + Clone + Send + Sync,
	TError: 'static + Error + Send + Sync,
{
	Box::pin(async move {
		if let Some(m) = nodes.clone().get(i) {
			// add populating the url parameters here
			let mut url_params = HashMap::new();
			if let Some(captures) = m.path_match.captures(&context.get_path()) {
				for var in m.path_match.capture_names() {
					if var.is_none() {
						continue;
					}
					let var = var.unwrap();
					let value = captures.name(var);
					if let Some(value) = value {
						url_params.insert(
							var.to_string(),
							value.as_str().to_string(),
						);
					}
				}
			}
			context.get_request_mut().params = url_params;
			m.handler
				.run_middleware(
					context,
					Box::new(move |context| {
						chained_run(context, nodes.clone(), i + 1)
					}),
				)
				.await
		} else {
			let method =
				context.get_request().hyper_request.method().to_string();
			let path = context.get_path();
			context
				.status(404)
				.map_err(TError::from_error)?
				.body(&format!("Cannot {} route {}", method, path))
				.await
				.map_err(TError::from_error)?;
			Ok(context)
		}
	})
}

pub struct App<
	TContext = DefaultContext,
	TMiddleware = DefaultMiddleware,
	TState = (),
	TError = DefaultError,
> where
	TContext: Context + Debug + Send + Sync,
	TMiddleware: Middleware<TContext, TError> + Clone + Send + Sync,
	TError: Error + Send + Sync,
	TState: Send + Sync,
{
	context_generator: ContextGeneratorFn<TContext, TState>,
	state: TState,
	pub(crate) error_handler: Option<ErrorHandlerFn<TError>>,

	get_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	post_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	put_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	delete_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	head_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	options_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	connect_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	patch_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
	trace_stack: Vec<MiddlewareHandler<TContext, TMiddleware, TError>>,
}

impl<TContext, TMiddleware, TState, TError>
	App<TContext, TMiddleware, TState, TError>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext, TError> + Clone + Send + Sync,
	TError: 'static + Error + Send + Sync,
	TState: Send + Sync,
{
	pub fn create(
		context_generator: ContextGeneratorFn<TContext, TState>,
		state: TState,
	) -> Self {
		App {
			context_generator,
			state,
			error_handler: None,

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

	pub fn get_state(&self) -> &TState {
		&self.state
	}

	pub fn set_error_handler(&mut self, error_handler: ErrorHandlerFn<TError>) {
		self.error_handler = Some(error_handler);
	}

	pub fn remove_error_handler(&mut self) {
		self.error_handler = None;
	}

	pub fn get<const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [TMiddleware; MIDDLEWARE_LENGTH],
	) {
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.get_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				true,
			));
			self.trace_stack
				.push(MiddlewareHandler::new(path, handler, true));
		});
	}

	pub fn post<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.post_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn put<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.put_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn delete<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.delete_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn head<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.head_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn options<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.options_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn connect<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.connect_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn patch<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.patch_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn trace<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			self.trace_stack.push(MiddlewareHandler::new(
				path,
				handler.into(),
				true,
			));
		});
	}

	pub fn use_middleware<IntoMiddleware, const MIDDLEWARE_LENGTH: usize>(
		&mut self,
		path: &str,
		middlewares: [IntoMiddleware; MIDDLEWARE_LENGTH],
	) where
		IntoMiddleware: Into<TMiddleware>,
	{
		IntoIterator::into_iter(middlewares).for_each(|handler| {
			let handler = handler.into();
			self.get_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.post_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.put_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.delete_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.head_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.options_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.connect_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.patch_stack.push(MiddlewareHandler::new(
				path,
				handler.clone(),
				false,
			));
			self.trace_stack
				.push(MiddlewareHandler::new(path, handler, false));
		});
	}

	pub fn use_sub_app<TSubAppState>(
		&mut self,
		base_path: &str,
		sub_app: App<TContext, TMiddleware, TSubAppState, TError>,
	) where
		TSubAppState: Send + Sync,
	{
		let base_path = {
			if base_path == "/" {
				"".to_string()
			} else {
				let mut formatted_base_path = base_path.to_string();

				// If it ends with /, remove it
				if let Some(stripped) = base_path.strip_suffix('/') {
					formatted_base_path = stripped.to_string();
				}

				// If it doesn't begin with a /, add it
				if !base_path.starts_with('/') {
					formatted_base_path = format!("/{}", base_path);
				}

				formatted_base_path
			}
		};

		self.get_stack
			.extend(sub_app.get_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.post_stack
			.extend(sub_app.post_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.put_stack
			.extend(sub_app.put_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.delete_stack
			.extend(sub_app.delete_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.head_stack
			.extend(sub_app.head_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.options_stack
			.extend(sub_app.options_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.connect_stack
			.extend(sub_app.connect_stack.into_iter().map(|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			}));

		self.patch_stack.extend(sub_app.patch_stack.into_iter().map(
			|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			},
		));

		self.trace_stack.extend(sub_app.trace_stack.into_iter().map(
			|handler| {
				MiddlewareHandler::new(
					&format!("{}{}", base_path, handler.mounted_url),
					handler.handler,
					handler.is_endpoint,
				)
			},
		));
	}

	pub async fn resolve(&self, context: TContext) -> Result<TContext, TError> {
		let method = context.get_method();
		let stack = self.get_middleware_stack(method, context.get_path());
		chained_run(context, Arc::new(stack), 0).await
	}

	pub(crate) fn generate_context(
		&self,
		request: Request,
		response: Response,
	) -> TContext {
		(self.context_generator)(request, response, self.get_state())
	}

	fn get_middleware_stack(
		&self,
		method: &HttpMethod,
		path: String,
	) -> Vec<MiddlewareHandler<TContext, TMiddleware, TError>> {
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
		};
		for handler in route_stack {
			if handler.is_match(&path) {
				stack.push(handler.clone());
			}
		}
		stack
	}
}

impl<TContext, TMiddleware, TState, TError> Default
	for App<TContext, TMiddleware, TState, TError>
where
	TContext: 'static + Context + Default + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext, TError> + Clone + Send + Sync,
	TError: 'static + Error + Send + Sync,
	TState: Default + Send + Sync,
{
	fn default() -> Self {
		Self::create(|_, _, _| TContext::default(), TState::default())
	}
}

impl<TContext, TMiddleware, TState, TError> Clone
	for App<TContext, TMiddleware, TState, TError>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext, TError> + Clone + Send + Sync,
	TError: 'static + Error + Send + Sync,
	TState: Clone + Send + Sync,
{
	fn clone(&self) -> Self {
		Self {
			context_generator: self.context_generator,
			state: self.state.clone(),
			error_handler: self.error_handler,

			get_stack: self.get_stack.clone(),
			post_stack: self.post_stack.clone(),
			put_stack: self.put_stack.clone(),
			delete_stack: self.delete_stack.clone(),
			head_stack: self.head_stack.clone(),
			options_stack: self.options_stack.clone(),
			connect_stack: self.connect_stack.clone(),
			patch_stack: self.patch_stack.clone(),
			trace_stack: self.trace_stack.clone(),
		}
	}
}
