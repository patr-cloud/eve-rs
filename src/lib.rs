#[macro_use]
extern crate async_trait;
extern crate async_std;
extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio;

mod app;
mod context;
mod cookie;
mod http_method;
mod middleware;
mod request;
mod response;

pub use app::App;
pub use context::Context;
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use http_method::HttpMethod;
pub use middleware::{Middleware, NextHandler};
pub use request::Request;
pub use response::Response;

use hyper::{
	service::{make_service_fn, service_fn},
	Body, Error, Response as HyperResponse, Server,
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
								.body(Body::from(response.response.clone()))
								.unwrap(),
						)
					}
				}))
			}
		});

		Server::bind(&bind_addr).serve(service).await.unwrap();
	}
	.await
}

#[derive(Clone)]
struct DefCtx {
	request: Request,
	response: Response,
}

impl Context for DefCtx {
	fn create(request: request::Request) -> Self {
		DefCtx {
			request,
			response: Response::new(),
		}
	}
	fn get_request(&self) -> &request::Request {
		&self.request
	}
	fn get_request_mut(&mut self) -> &mut request::Request {
		&mut self.request
	}
	fn get_response(&self) -> &response::Response {
		&self.response
	}
	fn get_response_mut(&mut self) -> &mut response::Response {
		&mut self.response
	}
}

#[derive(Clone)]
struct DefMdw {
	message: String,
}

#[async_trait]
impl Middleware<DefCtx> for DefMdw {
	async fn run(&self, mut context: DefCtx, next: NextHandler<DefCtx>) -> Result<DefCtx, Error> {
		println!("Pre: {}", self.message);
		context = next(context).await?;
		println!("Post: {}", self.message);
		Ok(context)
	}
}

/// Test code
#[test]
fn test_server() {
	let mut app = App::<DefCtx, DefMdw>::new();
	app.get(
		"/",
		&[
			DefMdw {
				message: "Test 1".to_owned(),
			},
			DefMdw {
				message: "Test 2".to_owned(),
			},
			DefMdw {
				message: "Test 3".to_owned(),
			},
		],
	);
	tokio::runtime::Runtime::new().unwrap().block_on(async {
		listen(app, ([127, 0, 0, 1], 3000)).await;
	});
}
