use crate::{Context, DefaultMiddleware, Error};
use serde_json::Value;
use std::fmt::Debug;

pub fn parser<TContext>(
	context: &TContext,
) -> Result<Option<Value>, Error<TContext>>
where
	TContext: 'static + Context + Debug + Send + Sync,
{
	if context.is(&["application/json"]) {
		let body = context
			.get_request()
			.get_body()
			.unwrap_or_else(|_| "None".to_string());
		let value = serde_json::from_str(&body)?;
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
