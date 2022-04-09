mod app;
mod routes;

use std::{
	net::{Ipv4Addr, SocketAddr},
	pin::Pin,
};

use eve_rs::{listen, App};
use futures::Future;

use crate::{
	app::{AppState, Ctx, Middler},
	routes::{create_todos, delete_todos, get_todos, update_todos},
};

macro_rules! middleware {
	( $ ($func: ident),+ ) => {
		[
            $ (
                Middler::new(|ctx, next| {
                    Box::pin(async move { $func(ctx, next).await })
                })
            ),+
        ]
	};
}

#[tokio::main]
async fn main() {
	println!("Starting server ...");

	let mut app = App::<Ctx, Middler, AppState, ()>::create(
		Ctx::new,
		AppState::default(),
	);

	app.get("/todos", middleware![get_todos]);
	app.post("/todos", middleware![create_todos]);
	app.patch("/todos/:id", middleware![update_todos]);
	app.delete("/todos/:id", middleware![delete_todos]);

	let bind_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 8080));
	println!("Listening on {bind_addr}");

	listen(
		app,
		bind_addr,
		None::<Pin<Box<dyn Future<Output = ()>>>>,
	)
	.await;
}
