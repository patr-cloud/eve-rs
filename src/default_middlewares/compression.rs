use crate::{Context, DefaultMiddleware};
use flate2::{Compress, Compression, FlushCompress, GzBuilder, Status};
use std::fmt::Debug;
use std::io::prelude::*;

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

	pub fn compress<TContext>(&mut self, context: &mut TContext)
	where
		TContext: Context + Debug + Send + Sync,
	{
		let allowed_encodings = context
			.get_request()
			.get_header("Accept-Encoding")
			.unwrap_or_else(|| String::new());
		let allowed_encodings = allowed_encodings
			.split(',')
			.map(str::trim)
			.collect::<Vec<&str>>();

		if allowed_encodings.contains(&"gzip") {
			let data = context.get_response().get_body();
			let mut output = vec![];
			let result = GzBuilder::new()
				.buf_read(data.as_ref(), self.compression_level)
				.read_to_end(&mut output);
			if let Ok(_) = result {
				context
					.body_bytes(&output)
					.header("Content-Encoding", "gzip");
			}
		} else if allowed_encodings.contains(&"deflate") {
			let data = context.get_response().get_body();
			let mut output = [];
			if let Ok(Status::Ok) =
				self.zlib_compressor
					.compress(&data, &mut output, FlushCompress::None)
			{
				context
					.body_bytes(&output)
					.header("Content-Encoding", "deflate");
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

			compressor.compress(&mut context);

			Ok(context)
		})
	})
}
