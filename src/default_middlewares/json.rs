use std::fmt::Debug;

use serde_json::Value;

use crate::{AsError, Context, DefaultMiddleware, Error};

pub async fn parser<TContext, TErrorData>(
	context: &mut TContext,
) -> Result<Option<Value>, Error<TErrorData>>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TErrorData: Default + Send + Sync,
	TContext::ResBodyBuffer: AsRef<[u8]>,
{
	if context.is(&["application/json"]) {
		let body = context.get_buffered_request_body().await;
		let value = serde_json::from_slice(body.as_ref())
			.status(400)
			.body("Bad request")?;
		Ok(Some(value))
	} else {
		Ok(None)
	}
}

pub fn default_parser<TData>() -> DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	DefaultMiddleware::new(|mut context, next| {
		Box::pin(async move {
			let json = parser(&mut context).await?;

			if let Some(json) = json {
				context.set_body_object(json);
			}

			next(context).await
		})
	})
}
