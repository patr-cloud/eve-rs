use std::{
	env,
	net::{Ipv4Addr, SocketAddr},
	pin::Pin,
};

use eve_rs::{
	default_middlewares::static_file_server::StaticFileServer,
	listen,
	App,
	DefaultContext,
	DefaultMiddleware,
	Request,
};
use futures::Future;

#[tokio::main]
async fn main() {
	println!("Starting server ...");

	let mut app = App::<DefaultContext, DefaultMiddleware<()>, (), ()>::create(
		|request: Request, _state: &()| DefaultContext::new(request),
		(),
	);

	app.get(
		"**",
		[DefaultMiddleware::new(move |context, next| {
			Box::pin(async move {
				StaticFileServer::create(
					env::current_dir().unwrap().to_str().unwrap(),
				)
				.serve(context, next)
				.await
			})
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
