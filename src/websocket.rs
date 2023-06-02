use std::{pin::Pin, sync::Arc};

use futures::{stream::SplitSink, SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use sha1::{Digest, Sha1};
use tokio::sync::Mutex;
pub use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
	tungstenite::protocol::Role,
	tungstenite::Error as TungsteniteError,
	WebSocketStream,
};

use crate::{error::EveError, Context};

#[derive(Clone)]
pub struct WebsocketConnection {
	writer: Arc<Mutex<SplitSink<WebSocketStream<Upgraded>, Message>>>,
}

impl WebsocketConnection {
	pub async fn send(&self, msg: Message) -> Result<(), TungsteniteError> {
		self.writer.lock().await.send(msg).await
	}

	pub async fn ping_connected(&self) -> Result<bool, TungsteniteError> {
		let result = self.writer.lock().await.send(Message::Ping(vec![])).await;
		match result {
			Ok(()) => Ok(true),
			Err(
				TungsteniteError::ConnectionClosed |
				TungsteniteError::AlreadyClosed,
			) => Ok(false),
			Err(err) => Err(err),
		}
	}
}

pub struct Websocket {
	on_connect: Option<
		Box<
			dyn FnOnce(
					WebsocketConnection,
				) -> Pin<
					Box<dyn std::future::Future<Output = ()> + Send + Sync>,
				> + Send
				+ Sync,
		>,
	>,
	on_disconnect: Option<
		Box<
			dyn FnOnce(
					WebsocketConnection,
				) -> Pin<
					Box<dyn std::future::Future<Output = ()> + Send + Sync>,
				> + Send
				+ Sync,
		>,
	>,
	on_message: Option<
		Box<
			dyn Fn(
					WebsocketConnection,
					Message,
				) -> Pin<
					Box<dyn std::future::Future<Output = ()> + Send + Sync>,
				> + Send
				+ Sync,
		>,
	>,
}

impl Websocket {
	pub fn new() -> Websocket {
		Websocket {
			on_connect: None,
			on_disconnect: None,
			on_message: None,
		}
	}

	pub fn on_connect<TFuture>(
		mut self,
		on_connect: impl FnOnce(WebsocketConnection) -> TFuture
			+ Send
			+ Sync
			+ 'static,
	) -> Self
	where
		TFuture: std::future::Future<Output = ()> + Send + Sync + 'static,
	{
		self.on_connect = Some(Box::new(move |conn| {
			Box::pin(async move { on_connect(conn).await })
		}));
		self
	}

	pub fn on_disconnect<TFuture>(
		mut self,
		on_disconnect: impl FnOnce(WebsocketConnection) -> TFuture
			+ Send
			+ Sync
			+ 'static,
	) -> Self
	where
		TFuture: std::future::Future<Output = ()> + Send + Sync + 'static,
	{
		self.on_disconnect = Some(Box::new(move |conn| {
			Box::pin(async move { on_disconnect(conn).await })
		}));
		self
	}

	pub fn on_message<TFuture>(
		mut self,
		on_message: impl Fn(WebsocketConnection, Message) -> TFuture
			+ Send
			+ Sync
			+ Clone
			+ 'static,
	) -> Self
	where
		TFuture: std::future::Future<Output = ()> + Send + Sync + 'static,
	{
		self.on_message = Some(Box::new(move |conn, message| {
			let on_message = on_message.clone();
			Box::pin(async move { on_message(conn, message).await })
		}));
		self
	}

	pub async fn connect<TContext>(
		self,
		mut context: TContext,
	) -> Result<TContext, EveError>
	where
		TContext: Context + Send,
	{
		if context
			.get_request_mut()
			.get_header(hyper::header::UPGRADE)
			.is_none()
		{
			context.status(400)?;
			return Ok(context);
		}

		let key = if let Some(key) =
			context.get_header(hyper::header::SEC_WEBSOCKET_KEY)
		{
			key
		} else {
			context.status(400)?;
			return Ok(context);
		};

		context
			.status(101)?
			.header(hyper::header::UPGRADE.as_str(), "websocket")?
			.header(hyper::header::CONNECTION.as_str(), "Upgrade")?
			.header(hyper::header::SEC_WEBSOCKET_VERSION.as_str(), "13")?
			.header(
				hyper::header::SEC_WEBSOCKET_ACCEPT.as_str(),
				&base64::encode({
					let mut sha1 = Sha1::new();
					sha1.update(key.as_bytes());
					sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
					sha1.finalize()
				}),
			)?;
		// Don't allow sending any more headers after the handshake is done
		context.get_response_mut().append_body_bytes(&[]).await?;

		match hyper::upgrade::on(
			context.get_request_mut().get_hyper_request_mut(),
		)
		.await
		{
			Ok(upgraded) => {
				let websocket = WebSocketStream::from_raw_socket(
					upgraded,
					Role::Server,
					None,
				)
				.await;
				let (ws_writer, mut ws_reader) = websocket.split();
				let websocket_connection = WebsocketConnection {
					writer: Arc::new(Mutex::new(ws_writer)),
				};
				if let Some(on_connect) = self.on_connect {
					on_connect(websocket_connection.clone()).await;
				}
				loop {
					match ws_reader.next().await {
						Some(Ok(message)) => match message {
							msg @ (Message::Text(_) | Message::Binary(_)) => {
								if let Some(on_message) =
									self.on_message.as_ref()
								{
									on_message(
										websocket_connection.clone(),
										msg,
									)
									.await;
								}
							}
							Message::Ping(data) => {
								if let Err(err) = websocket_connection
									.send(Message::Pong(data))
									.await
								{
									log::error!(
										"Unable to pong to websocket: {}",
										err
									);
								}
							}
							Message::Close(frame) => {
								if let Some(frame) = &frame {
									log::info!(
										"{} code: {}, reason: {}",
										"Closing websocket connection with",
										frame.code,
										frame.reason
									);
								}
								if let Err(err) = websocket_connection
									.send(Message::Close(frame))
									.await
								{
									log::error!(
										"Unable to close websocket: {}",
										err
									);
									break;
								}
							}
							_ => (),
						},
						Some(Err(err)) => {
							log::error!(
								"Unable to read from Websocket client: {}",
								err
							);
							break;
						}
						None => {
							log::warn!(
								"No websocket message from websocket client"
							);
							break;
						}
					}
				}
				if let Some(on_disconnect) = self.on_disconnect {
					on_disconnect(websocket_connection.clone()).await;
				}
				let _ = websocket_connection.writer.lock().await.close().await;
				Ok(context)
			}
			Err(err) => Err(EveError::RequestIo(err)),
		}
	}
}
