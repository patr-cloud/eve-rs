use crate::{
	context::{Context, DefaultContext},
	error::Error,
};
use std::{fmt::Debug, future::Future, pin::Pin};

pub type NextHandler<TContext> = Box<
	dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<TContext, Error>> + Send>> + Send + Sync,
>;

#[async_trait::async_trait]
pub trait Middleware<TContext: Context + Debug + Send + Sync> {
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error>;
}

type DefaultMiddlewareHandler =
	fn(
		DefaultContext,
		NextHandler<DefaultContext>,
	) -> Pin<Box<dyn Future<Output = Result<DefaultContext, Error>> + Send>>;

#[derive(Clone)]
pub struct DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	handler: DefaultMiddlewareHandler,
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

	pub fn new_with_data(handler: DefaultMiddlewareHandler, data: TData) -> Self {
		DefaultMiddleware { handler, data }
	}
}

#[async_trait::async_trait]
impl<TData> Middleware<DefaultContext> for DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: DefaultContext,
		next: NextHandler<DefaultContext>,
	) -> Result<DefaultContext, Error> {
		(self.handler)(context, next).await
	}
}
