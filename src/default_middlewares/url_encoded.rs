use std::fmt::Debug;

use serde_json::Value;

use crate::{AsError, Context, DefaultMiddleware, Error};

pub async fn parser<TContext, TErrorData>(
	context: &mut TContext,
) -> Result<Option<Value>, Error<TErrorData>>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TErrorData: Default + Send + Sync,
	TContext::ResBodyBuffer: AsRef<[u8]>
{
	if context.is(&["application/x-www-form-urlencoded"]) {
		let body = context.get_buffered_request_body().await;
		Ok(Some(
			serde_urlencoded::from_bytes(body.as_ref())
				.status(500)
				.body("Internal server error")?,
		))
	} else {
		Ok(None)
	}
}

pub fn default_parser<TData>() -> DefaultMiddleware<TData>
where
	TData: Default + Send + Clone + Sync,
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
