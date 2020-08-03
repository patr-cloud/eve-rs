use crate::{
	context::{Context, DefaultContext},
	error::Error,
};
use std::{fmt::Debug, future::Future, pin::Pin};

pub type NextHandler<TContext> = Box<
	dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<TContext, Error<TContext>>> + Send>>
		+ Send
		+ Sync,
>;

#[async_trait]
pub trait Middleware<TContext: Context + Debug + Clone + Send + Sync> {
	async fn run(
		&self,
		context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error<TContext>>;
}

type DefaultMiddlewareHandler =
	fn(
		DefaultContext,
		NextHandler<DefaultContext>,
	) -> Pin<Box<dyn Future<Output = Result<DefaultContext, Error<DefaultContext>>> + Send>>;

#[derive(Clone)]
pub struct DefaultMiddleware {
	handler: DefaultMiddlewareHandler,
}

impl DefaultMiddleware {
	pub fn new(handler: DefaultMiddlewareHandler) -> Self {
		DefaultMiddleware { handler }
	}
}

#[async_trait]
impl Middleware<DefaultContext> for DefaultMiddleware {
	async fn run(
		&self,
		context: DefaultContext,
		next: NextHandler<DefaultContext>,
	) -> Result<DefaultContext, Error<DefaultContext>> {
		(self.handler)(context, next).await
	}
}
