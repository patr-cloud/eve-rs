use std::{fmt::Debug, future::Future, pin::Pin};

use crate::{
	context::{Context, DefaultContext},
	error::{DefaultError, Error},
};

pub type NextHandler<TContext = DefaultContext, TError = DefaultError> = Box<
	dyn Fn(
			TContext,
		) -> Pin<Box<dyn Future<Output = Result<TContext, TError>> + Send>>
		+ Send
		+ Sync,
>;

#[async_trait::async_trait]
pub trait Middleware<TContext, TError>
where
	TContext: Context + Debug + Send + Sync,
	TError: Error + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext, TError>,
	) -> Result<TContext, TError>;
}

type DefaultMiddlewareHandler = fn(
	DefaultContext,
	NextHandler<DefaultContext, DefaultError>,
) -> Pin<
	Box<dyn Future<Output = Result<DefaultContext, DefaultError>> + Send>,
>;

#[derive(Clone)]
pub struct DefaultMiddleware<TData = ()>
where
	TData: Default + Clone + Send + Sync,
{
	handler: DefaultMiddlewareHandler,
	#[allow(dead_code)]
	data: TData,
}

impl<TData> DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	pub fn new(handler: DefaultMiddlewareHandler) -> Self {
		DefaultMiddleware {
			handler,
			data: Default::default(),
		}
	}

	pub fn new_with_data(
		handler: DefaultMiddlewareHandler,
		data: TData,
	) -> Self {
		DefaultMiddleware { handler, data }
	}

	pub fn get_data(&self) -> &TData {
		&self.data
	}
}

#[async_trait::async_trait]
impl<TData> Middleware<DefaultContext, DefaultError>
	for DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: DefaultContext,
		next: NextHandler<DefaultContext, DefaultError>,
	) -> Result<DefaultContext, DefaultError> {
		(self.handler)(context, next).await
	}
}
