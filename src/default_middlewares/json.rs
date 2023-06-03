use std::fmt::Debug;

use serde_json::Value;

use crate::{AsError, Context, DefaultMiddleware, Error};

pub fn parser<TContext, TErrorData>(
	context: &TContext,
) -> Result<Option<Value>, Error<TErrorData>>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TErrorData: Default + Send + Sync,
{
	if context.is(&["application/json"]) {
		let body = context
			.get_request()
			.get_body()
			.unwrap_or_else(|_| "None".to_string());
		let value = serde_json::from_str(&body)
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
			let json = parser(&context)?;

			if let Some(json) = json {
				context.set_body_object(json);
			}

			next(context).await
		})
	})
}
