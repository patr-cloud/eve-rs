use std::{fmt::Debug, io::prelude::*};

use flate2::{Compress, Compression, FlushCompress, GzBuilder, Status};

use crate::{Context, DefaultMiddleware};

pub const DEFAULT_COMPRESSION_LEVEL: u32 = 6;

pub struct CompressionHandler {
	zlib_compressor: Compress,
	compression_level: Compression,
}

impl CompressionHandler {
	pub fn create(compression_level: u32) -> CompressionHandler {
		let compression_level = Compression::new(compression_level);
		CompressionHandler {
			zlib_compressor: Compress::new(compression_level, false),
			compression_level,
		}
	}

	// TODO: Need to use async-compression crate for compressing on-demand
	pub async fn compress<TContext>(&mut self, context: &mut TContext)
	where
		TContext: Context + Debug + Send + Sync,
	{
		let allowed_encodings = context
			.get_request()
			.get_header("Accept-Encoding")
			.unwrap_or_else(String::new);
		let allowed_encodings = allowed_encodings
			.split(',')
			.map(str::trim)
			.collect::<Vec<&str>>();

		if allowed_encodings.contains(&"gzip") {
			let data =
				hyper::body::to_bytes(context.get_response_mut().take_body())
					.await
					.unwrap();
			let mut output = vec![];
			let result = GzBuilder::new()
				.buf_read(data.as_ref(), self.compression_level)
				.read_to_end(&mut output);
			if result.is_ok() {
				context
					.header("Content-Encoding", "gzip")
					.get_response_mut()
					.set_body(output);
			}
		} else if allowed_encodings.contains(&"deflate") {
			let data =
				hyper::body::to_bytes(context.get_response_mut().take_body())
					.await
					.unwrap();
			let mut output = vec![];
			if let Ok(Status::Ok) = self.zlib_compressor.compress(
				data.as_ref(),
				&mut output,
				FlushCompress::None,
			) {
				context
					.header("Content-Encoding", "deflate")
					.get_response_mut()
					.set_body(output);
			}
		}
	}
}

pub fn compression() -> CompressionHandler {
	CompressionHandler::create(DEFAULT_COMPRESSION_LEVEL)
}

pub fn compression_with_level(compression_level: u32) -> CompressionHandler {
	CompressionHandler::create(compression_level)
}

pub fn default_compression<TData>() -> DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	DefaultMiddleware::new(|mut context, next| {
		Box::pin(async move {
			let mut compressor = compression();

			context = next(context).await?;

			compressor.compress(&mut context).await;

			Ok(context)
		})
	})
}
