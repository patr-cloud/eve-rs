use std::{fmt::Debug, future::Future, pin::Pin};

use crate::{
	context::{Context, DefaultContext},
	error::{DefaultError, Error},
};

pub type NextHandler<TContext, TErrorData> = Box<
	dyn Fn(
			TContext,
		) -> Pin<
			Box<
				dyn Future<Output = Result<TContext, Error<TErrorData>>> + Send,
			>,
		> + Send
		+ Sync,
>;

#[async_trait::async_trait]
pub trait Middleware<TContext, TErrorData>
where
	TContext: Context + Debug + Send + Sync,
	TErrorData: Default + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext, TErrorData>,
	) -> Result<TContext, Error<TErrorData>>;
}

type DefaultMiddlewareHandler = fn(
	DefaultContext,
	NextHandler<DefaultContext, ()>,
) -> Pin<
	Box<dyn Future<Output = Result<DefaultContext, DefaultError>> + Send>,
>;

#[derive(Clone)]
pub struct DefaultMiddleware<TData>
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
impl<TData> Middleware<DefaultContext, ()> for DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: DefaultContext,
		next: NextHandler<DefaultContext, ()>,
	) -> Result<DefaultContext, DefaultError> {
		(self.handler)(context, next).await
	}
}
