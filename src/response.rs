use std::collections::HashMap;

pub struct Response {
	pub(crate) response: Vec<u8>,
	pub(crate) status: u16,
	pub(crate) headers: HashMap<String, Vec<String>>,
}

impl Response {
	pub fn new() -> Self {
		Response {
			response: vec![],
			status: 200,
			headers: HashMap::new(),
		}
	}

	pub fn set_status(&mut self, code: u16) {
		self.status = code;
	}

	pub fn set_header(&mut self, key: String, value: String) {
		if let Some(headers) = self.headers.get_mut(&key) {
			headers.push(value);
		} else {
			self.headers.insert(key, vec![value]);
		}
	}

	pub fn set_body(&mut self, data: &str) {
		self.response = data.as_bytes().to_vec();
	}

	pub fn set_body_bytes(&mut self, data: &[u8]) {
		self.response = data.to_vec();
	}
}
