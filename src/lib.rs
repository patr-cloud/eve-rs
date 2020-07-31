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
mod routeable;

pub use app::App;
pub use context::Context;
pub use cookie::{Cookie, CookieOptions, SameSite};
pub use http_method::HttpMethod;
pub use middleware::{Middleware, NextHandler};
pub use request::Request;
pub use response::Response;
pub use routeable::Routeable;

use hyper::{
	service::{make_service_fn, service_fn},
	Body, Error, Response as HyperResponse, Server,
};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};

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
				Ok::<_, Infallible>(service_fn(move |req: hyper::Request<Body>| {
					let app = app.clone();
					async move {
						let request = Request::from_hyper(req).await;
						let context = TContext::create(request);

						// execute app's middlewares
						let _response = app.resolve(context).await;

						Ok::<_, Infallible>(HyperResponse::new(Body::default()))
					}
				}))
			}
		});

		Server::bind(&bind_addr).serve(service).await.unwrap();
	}
	.await
}

#[derive(Clone)]
struct DefCtx;

impl Context for DefCtx {
	fn create(_request: request::Request) -> Self {
		todo!()
	}
	fn get_request(&self) -> &request::Request {
		todo!()
	}
	fn get_request_mut(&mut self) -> &mut request::Request {
		todo!()
	}
	fn get_response(&self) -> &response::Response {
		todo!()
	}
	fn get_response_mut(&mut self) -> &mut response::Response {
		todo!()
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
	let app = App::<DefCtx, DefMdw>::new();
	async_std::task::block_on(app.resolve(
		DefCtx,
		// vec![
		// 	MiddlewareHandler::new(
		// 		"/".to_owned(),
		// 		DefMdw {
		// 			message: "Test 1".to_owned(),
		// 		},
		// 	),
		// 	MiddlewareHandler::new(
		// 		"/".to_owned(),
		// 		DefMdw {
		// 			message: "Test 2".to_owned(),
		// 		},
		// 	),
		// 	MiddlewareHandler::new(
		// 		"/".to_owned(),
		// 		DefMdw {
		// 			message: "Test 3".to_owned(),
		// 		},
		// 	),
		// ],
	))
	.unwrap();

	return;
	let original_path = "/app/:applicationId\\.:version/changelog";

	let path = regex::Regex::new(":(?P<var>([a-zA-Z0-9_]+))")
		.unwrap()
		.replace_all(original_path, "(?P<$var>([a-zA-Z0-9_\\.-]+))");

	println!("{}", path);
	let request_url = "/app/kai.control.center.12345/changelog";
	let re = regex::Regex::new(&path).unwrap();
	let captures = re.captures(request_url).unwrap();
	for name in re.capture_names() {
		if name.is_none() {
			continue;
		}
		println!("{:#?}", name.unwrap());
	}
	for var in captures.iter() {
		println!("{:#?}", var);
	}

	//start_server(3000).await;
}

// /app/(?P<applicationId>([a-zA-Z0-9_-]+))-(?P<version>([a-zA-Z0-9_-]+))/changelog
