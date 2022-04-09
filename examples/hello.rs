use std::{
	net::{Ipv4Addr, SocketAddr},
	pin::Pin,
};

use eve_rs::{
	listen,
	App,
	Context,
	DefaultContext,
	DefaultMiddleware,
	Error,
	NextHandler,
	Request,
};
use futures::Future;

async fn say_hello(
	mut context: DefaultContext,
	_: NextHandler<DefaultContext, ()>,
) -> Result<DefaultContext, Error<()>> {
	let val = "Hello 👋";
	context.body(val);
	context.content_type("application/html");
	Ok(context)
}

#[tokio::main]
async fn main() {
	println!("Starting server ...");

	let mut app = App::<DefaultContext, DefaultMiddleware<()>, (), ()>::create(
		|request: Request, _state: &()| DefaultContext::new(request),
		(),
	);

	app.get(
		"/hello",
		[DefaultMiddleware::new(|context, next| {
			Box::pin(async move { say_hello(context, next).await })
		})],
	);

	let bind_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 8080));
	println!("Listening for connection on port {bind_addr}");

	listen(
		app,
		bind_addr,
		None::<Pin<Box<dyn Future<Output = ()>>>>,
	)
	.await;
}

#[allow(dead_code)]
async fn shutdown_signal() {
	tokio::signal::ctrl_c()
		.await
		.expect("Expected installing Ctrl+C signal handler");

	// Handle cleanup tasks
	println!("Shutting down the server !!!");
}
