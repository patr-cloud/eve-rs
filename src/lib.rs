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
	body,
	service::{make_service_fn, service_fn},
	Body, Error, Response as HyperResponse, Server, Version,
};
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};

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
				Ok::<_, Infallible>(service_fn(|req: hyper::Request<Body>| async {
					let (parts, body) = req.into_parts();
					let mut headers = HashMap::<String, Vec<String>>::new();
					parts.headers.iter().for_each(|(key, value)| {
						let key = key.to_string();
						let value = value.to_str();

						if value.is_err() {
							return;
						}
						let value = value.unwrap().to_string();

						if let Some(values) = headers.get_mut(&key) {
							values.push(value);
						} else {
							headers.insert(key.to_string(), vec![value]);
						}
					});
					let request = Request {
						body: body::to_bytes(body).await.unwrap().to_vec(),
						method: HttpMethod::from(parts.method),
						path: parts.uri.path().to_string(),
						version: match parts.version {
							Version::HTTP_09 => (0, 9),
							Version::HTTP_10 => (1, 0),
							Version::HTTP_11 => (1, 1),
							Version::HTTP_2 => (2, 0),
							Version::HTTP_3 => (3, 0),
							_ => (0, 0),
						},
						headers: headers.clone(),
						query: if let Some(query) = parts.uri.query() {
							query
								.split('&')
								.filter_map(|kv| {
									if kv.contains('=') {
										None
									} else {
										let mut items = kv
											.split('=')
											.map(String::from)
											.collect::<Vec<String>>();
										if items.len() != 2 {
											None
										} else {
											Some((items.remove(0), items.remove(1)))
										}
									}
								})
								.collect::<HashMap<String, String>>()
						} else {
							HashMap::new()
						},
						params: HashMap::new(),
						cookies: {
							let cookies_headers = headers.remove("Cookie");
							if let Some(header) = cookies_headers {
								let mut cookies = vec![];
								header.into_iter().for_each(|header| {
									// TODO
								});
								cookies
							} else {
								vec![]
							}
						},
					};

					// TODO execute app's middlewares

					let response = HyperResponse::new(Body::default());
					Ok::<HyperResponse<Body>, Infallible>(response)
				}))
			}
		});

		let server = Server::bind(&bind_addr).serve(service);
		server.await.unwrap();
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
