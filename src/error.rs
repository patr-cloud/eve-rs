use std::{error::Error as StdError, fmt::Debug};

use crate::Context;

#[derive(Debug)]
pub struct Error<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	pub(crate) context: Option<TContext>,
	pub(crate) message: String,
	pub(crate) status: u16,
	pub(crate) error: Box<dyn StdError + Send>,
}

impl<TContext> Error<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	pub fn new(
		context: Option<TContext>,
		message: String,
		status: u16,
		error: Box<dyn StdError + Send>,
	) -> Self {
		Error {
			context,
			message,
			status,
			error,
		}
	}

	pub fn get_context(&mut self) -> Option<&mut TContext> {
		self.context.as_mut()
	}
}

impl<TContext, StdErr> From<StdErr> for Error<TContext>
where
	TContext: Context + Debug + Send + Sync,
	StdErr: 'static + StdError + Send,
{
	fn from(err: StdErr) -> Self {
		Error {
			context: None,
			message: String::from("Internal Server Error"),
			status: 500,
			error: Box::new(err),
		}
	}
}
