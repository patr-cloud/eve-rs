use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Response {
	pub(crate) body: Vec<u8>,
	pub(crate) status: u16,
	pub(crate) headers: HashMap<String, Vec<String>>,
}

impl Response {
	pub fn new() -> Self {
		Response {
			body: vec![],
			status: 200,
			headers: HashMap::new(),
		}
	}

	pub fn get_status(&self) -> u16 {
		self.status
	}
	pub fn set_status(&mut self, code: u16) {
		self.status = code;
	}

	pub fn get_headers(&self) -> &HashMap<String, Vec<String>> {
		&self.headers
	}
	pub fn set_header(&mut self, key: String, value: String) {
		if let Some(headers) = self.headers.get_mut(&key) {
			headers.push(value);
		} else {
			self.headers.insert(key, vec![value]);
		}
	}

	pub fn get_body(&self) -> &Vec<u8> {
		&self.body
	}
	pub fn set_body(&mut self, data: &str) {
		self.body = data.as_bytes().to_vec();
	}
	pub fn set_body_bytes(&mut self, data: &[u8]) {
		self.body = data.to_vec();
	}
}

impl Default for Response {
	fn default() -> Self {
		Response {
			body: vec![],
			status: 200,
			headers: HashMap::new(),
		}
	}
}
