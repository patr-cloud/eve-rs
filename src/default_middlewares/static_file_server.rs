use std::fmt::Debug;

use futures::TryFutureExt;
use hyper::Body;
use tokio::fs::{self, File};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{Context, Error, Middleware, NextHandler};

#[derive(Clone)]
pub struct StaticFileServer {
	folder_path: String,
}

impl StaticFileServer {
	pub fn create(folder_path: &str) -> StaticFileServer {
		let folder_path = if let Some(stripped) = folder_path.strip_suffix('/')
		{
			stripped
		} else {
			folder_path
		}
		.to_string();
		StaticFileServer { folder_path }
	}

	pub async fn serve<TContext, TErrorData>(
		&self,
		mut context: TContext,
		next: NextHandler<TContext, TErrorData>,
	) -> Result<TContext, Error<TErrorData>>
	where
		TContext: Context + Debug + Send + Sync,
		TErrorData: Default + Send + Sync,
	{
		let file_location =
			format!("{}{}", self.folder_path, context.get_path());
		if is_file(&file_location).await {
			let stream = File::open(file_location)
				.map_ok(|file| FramedRead::new(file, BytesCodec::new()))
				.await.expect("File open should succeed as it has already met precondition");

			context
				.get_response_mut()
				.set_body(Body::wrap_stream(stream));

			Ok(context)
		} else {
			next(context).await
		}
	}
}

#[async_trait::async_trait]
impl<TContext, TErrorData> Middleware<TContext, TErrorData> for StaticFileServer
where
	TContext: 'static + Context + Debug + Send + Sync,
	TErrorData: 'static + Default + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext, TErrorData>,
	) -> Result<TContext, Error<TErrorData>> {
		self.serve(context, next).await
	}
}

pub fn static_server(folder_path: &str) -> StaticFileServer {
	StaticFileServer::create(folder_path)
}

async fn is_file(path: &str) -> bool {
	let metadata = fs::metadata(path).await;
	if metadata.is_err() {
		return false;
	}
	metadata.unwrap().is_file()
}
