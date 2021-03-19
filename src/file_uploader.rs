use hyper::body::HttpBody;

use crate::request::HyperRequest;

pub struct Field {
	name: String,
	request: HyperRequest,
}
