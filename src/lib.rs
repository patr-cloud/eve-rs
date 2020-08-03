#[macro_use]
extern crate async_trait;
extern crate async_std;
extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;

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

pub use app::App;
pub use context::{Context, DefaultContext};
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use error::Error;
pub use http_method::HttpMethod;
pub use middleware::{DefaultMiddleware, Middleware, NextHandler};
pub use request::Request;
pub use response::Response;

use async_std::net::TcpListener;
use hyper::{
	service::{make_service_fn, service_fn},
	Body, Error as HyperError, Response as HyperResponse, Server,
};
use std::{fmt::Debug, net::SocketAddr, sync::Arc};

pub async fn listen<TContext, TMiddleware>(
	app: App<TContext, TMiddleware>,
	bind_addr: ([u8; 4], u16),
) where
	TContext: Context + Debug + Clone + Send + Sync,
	TMiddleware: Middleware<TContext> + Clone + Send + Sync,
{
	let bind_addr = SocketAddr::from(bind_addr);

	let app_arc = Arc::new(app);

	async move {
		let service = make_service_fn(|_| {
			let app = app_arc.clone();

			async {
				Ok::<_, HyperError>(service_fn(move |req: hyper::Request<Body>| {
					let app = app.clone();
					async move {
						let request = Request::from_hyper(req).await;
						let context = TContext::create(request);

						// execute app's middlewares
						let result = app.resolve(context).await;
						let context = match result {
							Ok(context) => context,
							Err(err) => {
								// TODO return a proper formatted error
								return Ok::<_, HyperError>(HyperResponse::new(Body::from(
									err.message,
								)));
							}
						};
						let response = context.get_response();

						let mut hyper_response = HyperResponse::builder();

						// Set the appropriate headers
						for (key, values) in &response.headers {
							for value in values {
								hyper_response = hyper_response.header(key, value);
							}
						}

						Ok::<_, HyperError>(
							hyper_response
								.status(response.status)
								.body(Body::from(response.body.clone()))
								.unwrap(),
						)
					}
				}))
			}
		});

		let tcp_listener = TcpListener::bind(&bind_addr).await.unwrap();
		Server::builder(async_compat::HyperListener(tcp_listener))
			.executor(async_compat::HyperExecutor)
			.serve(service)
			.await
			.unwrap();
	}
	.await
}
