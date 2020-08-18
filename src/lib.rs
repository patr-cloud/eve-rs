#[macro_use]
extern crate async_trait;
extern crate async_std;
extern crate chrono;
extern crate flate2;
extern crate hyper;
extern crate log;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;

mod app;
mod async_compat;
mod context;
mod cookie;
mod error;
mod http_method;
mod middleware;
mod middleware_handler;
mod request;
mod response;

pub mod default_middlewares;

pub use app::App;
pub use context::{default_context_generator, Context, DefaultContext};
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use error::Error;
pub use http_method::HttpMethod;
pub use middleware::{DefaultMiddleware, Middleware, NextHandler};
pub use request::Request;
pub use response::Response;

use async_compat::HyperStream;
use async_std::net::TcpListener;
use futures::{channel::oneshot::Receiver, future};
use hyper::{
	service::{make_service_fn, service_fn},
	Body,
	Error as HyperError,
	Request as HyperRequest,
	Response as HyperResponse,
	Server,
};
use std::{fmt::Debug, net::SocketAddr, sync::Arc};

pub async fn listen<TContext, TMiddleware, TState>(
	app: App<TContext, TMiddleware, TState>,
	bind_addr: ([u8; 4], u16),
	shutdown_signal: Option<Receiver<()>>,
) where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext> + Clone + Send + Sync,
	TState: 'static + Send + Sync,
{
	let bind_addr = SocketAddr::from(bind_addr);

	let app_arc = Arc::new(app);

	async move {
		let service = make_service_fn(|conn: &HyperStream| {
			let app = app_arc.clone();
			let remote_addr = conn
				.0
				.peer_addr()
				.unwrap_or_else(|_| ([0, 0, 0, 0], 0).into());

			async move {
				Ok::<_, HyperError>(service_fn(move |req: HyperRequest<Body>| {
					let app = app.clone();
					async move {
						let request = Request::from_hyper(remote_addr, req).await;
						let mut context = app.generate_context(request);
						context.header("Server", "Eve");

						// execute app's middlewares
						let result = app.resolve(context).await;
						let response = match result {
							Ok(context) => context.take_response(),
							Err(err) => {
								// return a proper formatted error, if an error handler exists
								if app.error_handler.is_none() {
									return Ok::<_, HyperError>(HyperResponse::new(Body::from(
										err.message,
									)));
								} else {
									let response = Response::new();
									(app.error_handler.as_ref().unwrap())(response, err.error)
								}
							}
						};

						let mut hyper_response = HyperResponse::builder();

						// Set the appropriate headers
						for (key, values) in &response.headers {
							for value in values {
								hyper_response = hyper_response.header(key, value);
							}
						}

						Ok::<HyperResponse<Body>, HyperError>(
							hyper_response
								.status(response.status)
								.body(Body::from(response.body))
								.unwrap(),
						)
					}
				}))
			}
		});

		let tcp_listener = TcpListener::bind(&bind_addr).await.unwrap();
		let server = Server::builder(async_compat::HyperListener(tcp_listener))
			.executor(async_compat::HyperExecutor)
			.serve(service);

		if let Some(shutdown_signal) = shutdown_signal {
			server
				.with_graceful_shutdown(async {
					if let Err(_) = shutdown_signal.await {
						// TODO expose this error to the user
						future::pending::<()>().await;
					}
				})
				.await
				.unwrap();
		} else {
			server.await.unwrap();
		}
	}
	.await
}
