use std::{fmt::Debug, path::Path};

use futures::Stream;
use hyper::body::Bytes;
use multer::Multipart;
use s3::{creds::Credentials, Bucket};
use tokio::{fs, io};

pub use multer::{Constraints, Field};

use crate::{Context, Error, Middleware, NextHandler};

#[async_trait::async_trait]
pub trait UploadDestination<TContext>
where
	TContext: Context + Debug + Send + Sync + 'static,
{
	async fn handle_field(&self, file_name: &str, field: Field) -> Result<(), Error<TContext>>;
}

pub struct DirectoryUploadDestination {
	directory: String,
}

impl DirectoryUploadDestination {
	pub fn new(directory: &str) -> Self {
		Self {
			directory: String::from(directory),
		}
	}

	pub fn get_directory(&self) -> &str {
		self.directory.as_ref()
	}
}

#[async_trait::async_trait]
impl<TContext> UploadDestination<TContext> for DirectoryUploadDestination
where
	TContext: Context + Debug + Send + Sync + 'static,
{
	async fn handle_field(&self, file_name: &str, field: Field) -> Result<(), Error<TContext>> {
		let destination_path = Path::new(&self.directory).join(file_name);
		if !destination_path.exists() {
			fs::create_dir_all(&destination_path).await?;
		}
		fs::write(&destination_path, field.bytes().await?).await?;
		Ok(())
	}
}

pub struct S3UploadDestination {
	pub endpoint: String,
	pub region: String,
	pub bucket: String,
	pub key: String,
	pub secret: String,
	pub path_prefix: String,
}

#[async_trait::async_trait]
impl<TContext> UploadDestination<TContext> for S3UploadDestination
where
	TContext: Context + Debug + Send + Sync + 'static,
{
	async fn handle_field(&self, file_name: &str, field: Field) -> Result<(), Error<TContext>> {
		let credentials = Credentials::new(
			Some(self.key.as_str()),
			Some(self.secret.as_str()),
			None,
			None,
			None,
		)?;
		let bucket = Bucket::new(&self.bucket, self.region.parse()?, credentials)?;

		let (_, code) = bucket
			.put_object(
				format!("{}/{}", self.path_prefix, file_name),
				&field.bytes().await?,
			)
			.await?;
		if code == 200 {
			Ok(())
		} else {
			Err(Error {
				context: None,
				error: Box::new(io::Error::last_os_error()),
				message: format!("Upload to S3 failed with error code {}", code),
				status: code,
			})
		}
	}
}

pub struct FileUploader<TContext> {
	multipart: Multipart,
	upload_destination: Box<dyn UploadDestination<TContext> + Send + Sync>,
}

impl<TContext> FileUploader<TContext>
where
	TContext: Context + Debug + Send + Sync + 'static,
{
	pub fn new<TStream, TBytes, TError, TString, TUploadDestination>(
		stream: TStream,
		boundary: TString,
		upload_destination: TUploadDestination,
	) -> Self
	where
		TStream: Stream<Item = Result<TBytes, TError>> + Send + 'static,
		TBytes: Into<Bytes> + 'static,
		TError: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
		TString: Into<String>,
		TUploadDestination: UploadDestination<TContext> + Send + Sync + 'static,
	{
		let multipart = Multipart::new(stream, boundary);
		Self {
			multipart,
			upload_destination: Box::new(upload_destination),
		}
	}

	async fn new_with_constraints<TStream, TBytes, TError, TString, TUploadDestination>(
		stream: TStream,
		boundary: TString,
		upload_destination: TUploadDestination,
		constraints: Constraints,
	) -> Self
	where
		TStream: Stream<Item = Result<TBytes, TError>> + Send + 'static,
		TBytes: Into<Bytes> + 'static,
		TError: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
		TString: Into<String>,
		TUploadDestination: UploadDestination<TContext> + Send + Sync + 'static,
	{
		let multipart = Multipart::new_with_constraints(stream, boundary, constraints);
		Self {
			multipart,
			upload_destination: Box::new(upload_destination),
		}
	}
}

#[async_trait::async_trait]
impl<TContext> Middleware<TContext> for FileUploader<TContext>
where
	TContext: Context + Debug + Send + Sync + 'static,
{
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error<TContext>> {
		next(context).await
	}
}
