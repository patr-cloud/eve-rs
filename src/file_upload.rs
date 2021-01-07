// file that implements multipart.
use crate::context::Context;

use hyper::{header::CONTENT_TYPE, Body};
use multer::Multipart;
use std::fmt::Debug;

///function to parse incoming request for Multipart content type
///takes in 2 parameters
///1) context
///2) destination: path to store the incoming file.
pub async fn handle_multipart_request<TContext>(
	mut context: TContext,
	destination: &str,
) -> Result<(), &'static str>
where
	TContext: Context + Debug + Send + Sync,
{
	let request = context.get_request();

	//validate content type "multipart/form-data"
	let content_type = request.get_content_type();
	if !is_multipart_request(content_type) {
		//return from here
		return Err("not a multipart request.");
	}

	//get hyper request
	let hyper_request = request.get_hyper_request();
	let boundary = hyper_request
		.headers()
		.get(CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok());

	//since content type is already checked, boundry will not be null.
	//call request processer and return back response.
	if let Err(err) =
		process_multipart(boundary.unwrap(), hyper_request.into_body(), destination).await
	{
		return Err("Error occured while parsing the multipart request.");
	}
	Ok(())
}

// function to process multipart data
async fn process_multipart(boundary: String, body: Body, destination: &str) -> multer::Result<()> {
	// create multipart obj
	let multipart = Multipart::new(body, boundary);

	// iterate over the fields in multipart request.
	// get request body
	while let Some(mut field) = multipart.next_field().await? {
		let name = field.name();
		let file_name = field.file_name();
		let content_type = field.content_type();

		// store the files at destination dir.

		// for testing only
		println!(
			"Name {:?}, FileName: {:?}, Content-Type: {:?}",
			name, file_name, content_type
		);
	}
	Ok(())
}

// helper function.
// function to validate content type.
fn is_multipart_request(content_type: String) -> bool {
	if content_type == "multipart/form-data" {
		return true;
	}

	false
}
