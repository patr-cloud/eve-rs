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
mod http_method;
mod middleware;
mod middleware_handler;
mod request;
mod response;
mod error;

pub use app::App;
pub use context::{Context, DefaultContext};
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use http_method::HttpMethod;
pub use middleware::{Middleware, NextHandler, DefaultMiddleware};
pub use request::Request;
pub use response::Response;
pub use hyper::Error;

use async_std::net::TcpListener;
use hyper::{
	service::{make_service_fn, service_fn},
	Body, Response as HyperResponse, Server,
};
use std::{net::SocketAddr, sync::Arc};

pub async fn listen<TContext, TMiddleware>(
	app: App<TContext, TMiddleware>,
	bind_addr: ([u8; 4], u16),
) where
	TContext: Context + Clone + Send + Sync,
	TMiddleware: Middleware<TContext> + Clone + Send + Sync,
{
	let bind_addr = SocketAddr::from(bind_addr);

	let app_arc = Arc::new(app);

	async move {
		let service = make_service_fn(|_| {
			let app = app_arc.clone();

			async {
				Ok::<_, Error>(service_fn(move |req: hyper::Request<Body>| {
					let app = app.clone();
					async move {
						let request = Request::from_hyper(req).await;
						let context = TContext::create(request);

						// execute app's middlewares
						let context = app.resolve(context).await?;
						let response = context.get_response();

						let mut hyper_response = HyperResponse::builder();

						// Set the appropriate headers
						for (key, values) in &response.headers {
							for value in values {
								hyper_response = hyper_response.header(key, value);
							}
						}

						Ok::<_, Error>(
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
