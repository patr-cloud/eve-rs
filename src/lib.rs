mod app;
mod context;
mod cookie;
mod error;
mod http_method;
mod middleware;
mod middleware_handler;
mod request;
mod response;
// mod headers;
mod macros;
mod renderer;

// pub mod default_middlewares;
pub mod websocket;

use std::{convert::TryInto, fmt::Debug, net::SocketAddr, sync::Arc};

use futures::Future;
pub use handlebars;
use hyper::{
	server::conn::AddrStream,
	service::{make_service_fn, service_fn},
	Body,
	Error as HyperError,
	HeaderMap,
	Request as HyperRequest,
	Response as HyperResponse,
	Server,
};
use response::PreBodySenderData;
use tokio::{sync::mpsc, task};

pub use self::{
	app::App,
	context::{default_context_generator, Context, DefaultContext},
	cookie::{Cookie, CookieOptions, SameSite},
	error::{AsError, DefaultError, Error, EveError},
	http_method::HttpMethod,
	middleware::{DefaultMiddleware, Middleware, NextHandler},
	renderer::RenderEngine,
	request::Request,
	response::Response,
};

pub async fn listen<
	TContext,
	TMiddleware,
	TState,
	TError,
	TShutdownSignal,
	TListenAddr,
>(
	app: App<TContext, TMiddleware, TState, TError>,
	bind_addr: TListenAddr,
	shutdown_signal: Option<TShutdownSignal>,
) where
	TContext: 'static + Context + Debug + Send + Sync,
	TMiddleware: 'static + Middleware<TContext, TError> + Clone + Send + Sync,
	TState: 'static + Send + Sync,
	TShutdownSignal: Future<Output = ()>,
	TError: 'static + Error + Send + Sync,
	TListenAddr: Into<SocketAddr>,
{
	let app_arc = Arc::new(app);

	async move {
		let service = make_service_fn(|conn: &AddrStream| {
			let app = app_arc.clone();
			let remote_addr = conn.remote_addr();

			async move {
				Ok::<_, HyperError>(service_fn(
					move |hyper_request: HyperRequest<Body>| {
						let app = app.clone();
						async move {
							let method: HttpMethod =
								match hyper_request.method().try_into() {
									Ok(method) => method,
									Err(_) => {
										log::warn!(
											"Unknown method. Ignoring request"
										);
										return Ok(HyperResponse::new(
											Body::empty(),
										));
									}
								};
							let path = hyper_request.uri().path().to_string();
							let request = Request::new(
								remote_addr,
								method.clone(),
								hyper_request,
							);

							let (sender, mut receiver) =
								mpsc::unbounded_channel();

							task::spawn(async move {
								let response = Response::new(sender.clone());

								let mut context =
									app.generate_context(request, response);

								let _ = context.header("Server", "Eve");

								let _ = sender.send(
									PreBodySenderData::Status(200),
								);
								// execute app's middlewares
								let result = app.resolve(context).await;

								if let Err(err) = result {
									// return a proper formatted error, if an
									// error handler exists
									if let Some(handler) = app.error_handler {
										let response = Response::new(sender);
										handler(response, err)
											.await
											.unwrap_or_else(|err| {
												log::error!("Error occured running error handler: {}", err);
											});
									} else {
										let _ = sender.send(
											PreBodySenderData::Status(500),
										);
										let _ = sender.send(
											PreBodySenderData::Body(
												Body::from(
													"Internal server error",
												),
											),
										);
									}
								}
							});

							let mut status = None;
							let mut body = Body::from(format!(
								"Could not {} route {}",
								method, path
							));
							let mut headers = HeaderMap::new();

							while let Some(data) = receiver.recv().await {
								match data {
									PreBodySenderData::Status(value) => {
										status = Some(value);
									}
									PreBodySenderData::SetHeader(
										key,
										value,
									) => {
										headers.append(key, value);
									}
									PreBodySenderData::RemoveHeader(key) => {
										headers.remove(key);
									}
									PreBodySenderData::ClearHeaders => {
										headers.clear();
									}
									PreBodySenderData::Body(value) => {
										body = value;
										break;
									}
								}
							}

							let mut response_builder = HyperResponse::builder()
								.status(status.unwrap_or(404));
							for (key, value) in headers.iter() {
								response_builder =
									response_builder.header(key, value);
							}
							Ok::<HyperResponse<Body>, HyperError>(
								response_builder.body(body).unwrap_or_default(),
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
