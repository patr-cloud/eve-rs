use eve_rs::{
	default_middleware,
	websocket::Websocket,
	App,
	DefaultContext,
	DefaultError,
	Error,
	NextHandler,
};
use tokio_tungstenite::tungstenite::Message;

async fn respond(
	context: DefaultContext,
	_: NextHandler,
) -> Result<DefaultContext, DefaultError> {
	Websocket::new()
		.on_connect(|_| async {
			println!("Connection established!");
		})
		.on_disconnect(|_| async {
			println!("Connection closed");
		})
		.on_message(|connection, message| async move {
			println!("Message recieved from connection: {:?}", message);
			connection
				.send(Message::Text(
					message.into_text().unwrap().chars().rev().collect(),
				))
				.await
				.unwrap();
		})
		.connect(context)
		.await
		.map_err(DefaultError::from_error)
}

#[tokio::test]
async fn playground() {
	let mut app = App::create(eve_rs::default_context_generator, ());
	app.get("/", default_middleware!(respond));
	eve_rs::listen(app, ([0, 0, 0, 0], 3000), Some(futures::future::pending()))
		.await;
}
