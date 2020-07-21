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
pub use middleware::Middleware;
pub use routeable::Routeable;

/// Test code
#[test]
fn test_server() {
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
