use std::fmt::Debug;

use serde_json::Value;

use crate::{AsError, Context, DefaultMiddleware, Error};

pub fn parser<TContext, TError>(
	context: &TContext,
) -> Result<Option<Value>, TError>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TError: Error + Send + Sync,
{
	if context.is(&["application/x-www-form-urlencoded"]) {
		let body = context
			.get_request()
			.get_body()
			.unwrap_or_else(|_| "None".to_string());
		Ok(Some(
			serde_urlencoded::from_bytes(body.as_bytes())
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
			let json = parser(&context)?;

			if let Some(json) = json {
				context.set_body_object(json);
			}

			next(context).await
		})
	})
}
