use crate::context::{Context, DefaultContext};
use hyper::Error;
use std::{future::Future, pin::Pin};

pub type NextHandler<TContext> = Box<
	dyn Fn(TContext) -> Pin<Box<dyn Future<Output = Result<TContext, Error>> + Send>> + Send + Sync,
>;

#[async_trait]
pub trait Middleware<TContext: Context + Send + Sync> {
	async fn run(&self, context: TContext, next: NextHandler<TContext>) -> Result<TContext, Error>;
}

#[derive(Clone)]
pub struct DefaultMiddleware {
	handler: fn(
		DefaultContext,
		NextHandler<DefaultContext>,
	) -> Pin<Box<dyn Future<Output = Result<DefaultContext, Error>> + Send>>,
}

impl DefaultMiddleware {
	pub fn new(
		handler: fn(
			DefaultContext,
			NextHandler<DefaultContext>,
		) -> Pin<Box<dyn Future<Output = Result<DefaultContext, Error>> + Send>>,
	) -> Self {
		DefaultMiddleware {
			handler
		}
	}
}

#[async_trait]
impl Middleware<DefaultContext> for DefaultMiddleware {
	async fn run(
		&self,
		context: DefaultContext,
		next: NextHandler<DefaultContext>,
	) -> Result<DefaultContext, Error> {
		(self.handler)(context, next).await
	}
}
