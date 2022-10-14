use std::fmt::Debug;

use serde_json::Value;

use crate::{AsError, Context, DefaultError, DefaultMiddleware, Error};
use std::error::Error as StdError;

pub async fn parser<TContext, TError>(
	context: &mut TContext,
) -> Result<Option<Value>, TError>
where
	TContext: 'static + Context + Debug + Send + Sync,
	TError: Error + StdError + Send + Sync,
{
	if context.is(&["application/json"]) {
		let body = context
			.get_request_mut()
			.get_body()
			.await
			.map_err(TError::from_error)?;
		let value = serde_json::from_str(&body)
			.map_err(std::io::Error::from)
			.map_err(TError::from_error)
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
			let json = parser::<_, DefaultError>(&mut context).await?;

			if let Some(json) = json {
				context.set_body_object(json);
			}

			next(context).await
		})
	})
}
