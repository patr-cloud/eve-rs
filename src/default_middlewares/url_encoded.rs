use crate::{Context, DefaultMiddleware, Error};
use serde_json::Map;
use serde_json::Value;
use std::fmt::Debug;
use url::form_urlencoded::parse;

pub fn parser<TContext>(context: &TContext) -> Result<Option<Value>, Error<TContext>>
where
	TContext: 'static + Context + Debug + Clone + Send + Sync,
{
	if context.is(&["application/x-www-form-urlencoded"]) {
		let body = context
			.get_request()
			.get_body()
			.unwrap_or_else(|_| "None".to_string());
		Ok(Some(Value::Object(
			parse(body.as_bytes())
				.into_iter()
				.map(|(a, b)| (a.to_string(), Value::String(b.to_string())))
				.collect::<Map<String, Value>>(),
		)))
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
