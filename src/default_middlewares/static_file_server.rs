use crate::{Context, Error, NextHandler};
use async_std::{fs, path::Path};
use std::fmt::Debug;

pub struct StaticFileServer {
	folder_path: String,
}

impl StaticFileServer {
	pub fn create(folder_path: &str) -> StaticFileServer {
		let folder_path = if folder_path.ends_with("/") {
			&folder_path[..folder_path.len() - 1]
		} else {
			folder_path
		}
		.to_string();
		StaticFileServer { folder_path }
	}

	pub async fn serve<TContext>(
		&self,
		mut context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error<TContext>>
	where
		TContext: Context + Debug + Clone + Send + Sync,
	{
		let file_location = format!("{}{}", self.folder_path, context.get_path());
		let path = Path::new(&file_location);
		if path.exists().await && path.is_file().await {
			let content = fs::read(file_location).await?;
			context.body_bytes(&content);
			Ok(context)
		} else {
			next(context).await
		}
	}
}

pub fn static_server(folder_path: &str) -> StaticFileServer {
	StaticFileServer::create(folder_path)
}
