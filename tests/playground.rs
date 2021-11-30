use std::time::Duration;

use eve_rs::{App, Context, DefaultContext, DefaultMiddleware};

#[tokio::test]
async fn playground() {
	let mut app = App::<DefaultContext, DefaultMiddleware<()>, (), ()>::create(
		eve_rs::default_context_generator,
		(),
	);
	app.get(
		"/",
		[DefaultMiddleware::new(|mut context, _| {
			Box::pin(async move {
				let response = context.get_response_mut();
				response
					.append_body_bytes(b"first body data\n")
					.await
					.expect("unable to set first body");
				tokio::time::sleep(Duration::from_millis(5000)).await;
				response
					.append_body_bytes(b"second body data\n")
					.await
					.expect("unable to set second body");
				Ok(context)
			})
		})],
	);
	eve_rs::listen(app, ([0, 0, 0, 0], 3000), Some(futures::future::pending()))
		.await;
}
