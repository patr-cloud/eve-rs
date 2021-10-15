mod app;
mod context;
mod cookie;
mod error;
mod http_method;
mod middleware;
mod middleware_handler;
mod request;
mod response;
//mod headers;
#[cfg(feature = "render")]
mod renderer;

pub mod default_middlewares;

use std::{fmt::Debug, net::SocketAddr, sync::Arc};

pub use app::App;
pub use context::{default_context_generator, Context, DefaultContext};
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use error::{AsError, DefaultError, Error};
use futures::Future;
pub use handlebars;
pub use http_method::HttpMethod;
use hyper::{
	server::conn::AddrStream,
	service::{make_service_fn, service_fn},
	Body,
	Error as HyperError,
	Request as HyperRequest,
	Response as HyperResponse,
	Server,
	StatusCode,
};
pub use middleware::{DefaultMiddleware, Middleware, NextHandler};
pub use renderer::RenderEngine;
pub use request::Request;
pub use response::Response;

pub async fn listen<
	TContext,
	TMiddleware,
	TState,
	TErrorData,
	TShutdownSignal,
	TListenAddr,
>(
	app: App<TContext, TMiddleware, TState, TErrorData>,
	bind_addr: TListenAddr,
	shutdown_signal: Option<TShutdownSignal>,
) where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware:
		'static + Middleware<TContext, TErrorData> + Clone + Send + Sync,
	TState: 'static + Send + Sync,
	TShutdownSignal: Future<Output = ()>,
	TErrorData: 'static + Default + Send + Sync,
	TListenAddr: Into<SocketAddr>,
{
	let app_arc = Arc::new(app);

	async move {
		let service = make_service_fn(|conn: &AddrStream| {
			let app = app_arc.clone();
			let remote_addr = conn.remote_addr();

			async move {
				Ok::<_, HyperError>(service_fn(
					move |req: HyperRequest<Body>| {
						let app = app.clone();
						async move {
							let request =
								Request::from_hyper(remote_addr, req).await;
							let mut context = app.generate_context(request);
							context.header("Server", "Eve");

							// execute app's middlewares
							let result = app.resolve(context).await;
							let response = match result {
								Ok(context) => context.take_response(),
								Err(err) => {
									// return a proper formatted error, if an
									// error handler exists
									if let Some(handler) = app.error_handler {
										let response = Response::new();
										handler(response, err)
									} else {
										let mut hyper_response =
											HyperResponse::new(Body::from(
												Vec::from(
													err.get_body_bytes()
														.unwrap_or_else(|| {
															"Internal server error"
															.as_bytes()
														}),
												),
											));
										*hyper_response.status_mut() =
										StatusCode::from_u16(err.get_status().unwrap_or(500))
											.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
										return Ok(hyper_response);
									}
								}
							};

							let mut hyper_response = HyperResponse::builder();

							// Set the appropriate headers
							for (key, values) in &response.headers {
								for value in values {
									hyper_response =
										hyper_response.header(key, value);
								}
							}

							Ok::<HyperResponse<Body>, HyperError>(
								hyper_response
									.status(response.status)
									.body(Body::from(response.body))
									.unwrap(),
							)
						}
					},
				))
			}
		});

		let server = Server::bind(&bind_addr.into()).serve(service);

		if let Some(shutdown_signal) = shutdown_signal {
			server
				.with_graceful_shutdown(shutdown_signal)
				.await
				.unwrap();
		} else {
			server.await.unwrap();
		}
	}
	.await
}
