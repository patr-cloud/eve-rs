use crate::{listen, App, Context, DefaultContext, DefaultMiddleware, Error, NextHandler};

async fn main() {
	let mut app = App::new();

	app.use_middleware(
		"/",
		&[DefaultMiddleware::<()>::new(|context, next| {
			Box::pin(async { Ok(context) })
		})],
	);
	app.use_middleware(
		"/",
		&[DefaultMiddleware::<String>::new(|context, next| {
			Box::pin(async { Ok(context) })
		})],
	);

	listen(app, ([127, 0, 0, 1], 3000)).await;
}
