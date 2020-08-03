use std::{error::Error as StdError, fmt::Debug};

use crate::Context;

#[derive(Debug)]
pub struct Error<TContext>
where
	TContext: Context + Debug + Clone + Send + Sync,
{
	pub(crate) context: Option<TContext>,
	pub(crate) message: String,
	pub(crate) status: u16,
	pub(crate) error: Option<Box<dyn StdError>>,
}

impl<TContext> Error<TContext>
where
	TContext: Context + Debug + Clone + Send + Sync,
{
	pub fn new(
		context: Option<TContext>,
		message: String,
		status: u16,
		error: Option<Box<dyn StdError>>,
	) -> Self {
		Error {
			context,
			message,
			status,
			error,
		}
	}

	pub fn unauthorized(context: TContext) -> Self {
		Error {
			context: Some(context),
			message: String::from("Unauthorized"),
			status: 401,
			error: None,
		}
	}

	pub fn not_found(context: TContext) -> Self {
		Error {
			context: Some(context),
			message: String::from("Not Found"),
			status: 404,
			error: None,
		}
	}
}

impl<TContext, StdErr> From<StdErr> for Error<TContext>
where
	TContext: Context + Debug + Clone + Send + Sync,
	StdErr: 'static + StdError,
{
	fn from(err: StdErr) -> Self {
		Error {
			context: None,
			message: String::from("Internal Server Error"),
			status: 500,
			error: Some(Box::new(err)),
		}
	}
}
