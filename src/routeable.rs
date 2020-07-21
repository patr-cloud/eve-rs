use crate::{context::Context, middleware::Middleware};

pub trait Routeable<TContext: Context, TMiddleware: Middleware<TContext>> {
	fn get(&mut self, path: &str, middleware: TMiddleware);
	fn post(&mut self, path: &str, middleware: TMiddleware);
	fn put(&mut self, path: &str, middleware: TMiddleware);
	fn delete(&mut self, path: &str, middleware: TMiddleware);
	fn head(&mut self, path: &str, middleware: TMiddleware);
	fn options(&mut self, path: &str, middleware: TMiddleware);
	fn connect(&mut self, path: &str, middleware: TMiddleware);
	fn patch(&mut self, path: &str, middleware: TMiddleware);
	fn trace(&mut self, path: &str, middleware: TMiddleware);
	fn use_middleware(&mut self, path: &str, middleware: TMiddleware);
}
