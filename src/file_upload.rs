// file that implements multipart.
use crate::{cookie::Cookie, request::Request, response::Response, HttpMethod,
context::Context, error::Error};
use bytes::Bytes;

use std::{convert::Infallible, net::SocketAddr, fmt::Debug};
use hyper::http::request;
use multer::Multipart;
use futures::stream::once;
use futures::stream::Stream;

// function to parse incoming request for Multipart content type

async fn handle_multipart_request<TContext>(mut context : TContext
) -> Result<TContext, Error<TContext>>
where
TContext: Context + Debug + Send + Sync
{
	//get request from context
	let request = context.get_request();

	// extract multipart from boundry
	let boundry_string = request
		.get_content_type();

	let boundry = multer::parse_boundary(boundry_string).ok();

	// check if content type is not "multipart/form-data"

	if boundry.is_none() {
		// TODO: send bad request.
	}

	// check for error while parsing.
	// if let Err(err) = process_multipart(request)

	Ok(context)
}


// function to process multipart data
async fn process_multipart(request : Request, boundry : String) -> multer::Result<()> {

	// get request body
	let body = request.get_body();

	let data = "--X-BOUNDARY\r\nContent-Disposition: form-data; name=\"my_text_field\"\r\n\r\nabcd\r\n--X-BOUNDARY--\r\n";
	let temp_body_stream = once(async move {Result::<Bytes, Infallible>::Ok(Bytes::from(data))});
	
	let mut multipart = Multipart::new(temp_body_stream, boundry);

	






	Ok(())
}