use std::sync::Arc;

use handlebars::{Handlebars, RenderError};
use serde::Serialize;

use crate::Context;

pub trait RenderEngine: Context {
	fn get_register(&self) -> &Arc<Handlebars>;
	fn set_register(&mut self, register: Arc<Handlebars<'static>>);

	fn render<TParams>(
		&mut self,
		template_name: &str,
		data: &TParams,
	) -> Result<&mut Self, RenderError>
	where
		TParams: Serialize,
	{
		let rendered = self.get_register().render(template_name, data)?;
		self.body(&rendered);
		Ok(self)
	}
}
