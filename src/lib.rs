#[macro_use]
extern crate async_trait;
extern crate async_std;
extern crate hyper;
extern crate regex;
extern crate serde_json;
extern crate tokio;

mod app;
mod context;
mod http_method;
mod middleware;
mod routeable;

pub use app::App;
pub use context::Context;
pub use http_method::HttpMethod;
use hyper::Error;
pub use middleware::{Middleware, MiddlewareHandler, NextHandler};
pub use routeable::Routeable;

#[derive(Clone)]
struct DefCtx;

impl Context for DefCtx {
	fn get_raw_body(&self) -> &[u8] {
		todo!()
	}
	fn get_cookies(&self) -> Vec<context::Cookie> {
		todo!()
	}
	fn get_host(&self) -> &str {
		todo!()
	}
	fn get_ip(&self) -> &str {
		todo!()
	}
	fn get_ips(&self) -> Vec<&str> {
		todo!()
	}
	fn get_method(&self) -> &str {
		todo!()
	}
	fn get_path(&self) -> &str {
		todo!()
	}
	fn get_params(&self) -> std::collections::HashMap<String, String> {
		todo!()
	}
	fn get_protocol(&self) -> &str {
		todo!()
	}
	fn get_query(&self) -> std::collections::HashMap<String, serde_json::Value> {
		todo!()
	}
	fn get(&self, header: &str) -> Option<&str> {
		todo!()
	}
}

#[derive(Clone)]
struct DefMdw {
	message: String,
}

impl Middleware<DefCtx> for DefMdw {
	fn run(&self, mut context: DefCtx, next: NextHandler<DefCtx>) -> Result<DefCtx, Error> {
		println!("Pre: {}", self.message);
		context = next(context)?;
		println!("Post: {}", self.message);
		Ok(context)
	}
}

/// Test code
#[test]
fn test_server() {
	let app = App::<DefCtx, DefMdw>::new::<DefCtx, DefMdw>();
	app.resolve(
		DefCtx,
		vec![
			MiddlewareHandler::new(
				"/".to_owned(),
				DefMdw {
					message: "Test 1".to_owned(),
				},
			),
			MiddlewareHandler::new(
				"/".to_owned(),
				DefMdw {
					message: "Test 2".to_owned(),
				},
			),
			MiddlewareHandler::new(
				"/".to_owned(),
				DefMdw {
					message: "Test 3".to_owned(),
				},
			),
		],
	)
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
