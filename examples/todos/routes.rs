use std::str::FromStr;

use eve_rs::{Context, Error, NextHandler};
use uuid::Uuid;

use crate::app::{Ctx, Todo};

pub async fn get_todos(
	mut ctx: Ctx,
	_next: NextHandler<Ctx, ()>,
) -> Result<Ctx, Error<()>> {
	let todos = ctx
		.state
		.db
		.read()
		.expect("Expected to read DB data")
		.values()
		.cloned()
		.collect::<Vec<_>>();
    // TODO: Convert DB error into Error types


	ctx.body(
		&serde_json::to_string_pretty(&todos)
			.expect("Expected to serialize the todos to json string"),
	);
	ctx.content_type("application/json");
	ctx.status(200);

	Ok(ctx)
}

pub async fn create_todos(
	mut ctx: Ctx,
	_next: NextHandler<Ctx, ()>,
) -> Result<Ctx, Error<()>> {
	let text = ctx
		.get_request()
		.get_query()
		.get("text")
		.cloned()
		.expect("Expected text in query params");

	let todo = Todo {
		id: Uuid::new_v4(),
		text,
		completed: false,
	};

	ctx.state
		.db
		.write()
		.expect("Expected to write todo in DB")
		.insert(todo.id, todo.clone());

	ctx.body(
		&serde_json::to_string_pretty(&todo)
			.expect("Expected to serialize the todos to json string"),
	);
	ctx.content_type("application/json");
	ctx.status(201);

	Ok(ctx)
}

pub async fn update_todos(
	mut ctx: Ctx,
	_next: NextHandler<Ctx, ()>,
) -> Result<Ctx, Error<()>> {
	let id = ctx
		.get_request()
		.get_params()
		.get("id")
		.map(|id| Uuid::from_str(id).unwrap())
		.expect("Expected id in path param");

	let mut todo = ctx.state.db.read().unwrap().get(&id).cloned().unwrap();

	if let Some(text) = ctx.get_request().get_query().get("text").cloned() {
		todo.text = text;
	};

	if let Some(completed) =
		ctx.get_request().get_query().get("completed").cloned()
	{
		todo.completed = bool::from_str(&completed).unwrap();
	};

	ctx.state
		.db
		.write()
		.expect("Expected to write todo in DB")
		.insert(todo.id, todo.clone());

	ctx.body(
		&serde_json::to_string_pretty(&todo)
			.expect("Expected to serialize the todos to json string"),
	);
	ctx.content_type("application/json");
	ctx.status(201);

	Ok(ctx)
}

pub async fn delete_todos(
	mut ctx: Ctx,
	_next: NextHandler<Ctx, ()>,
) -> Result<Ctx, Error<()>> {
	let id = ctx
		.get_request()
		.get_params()
		.get("id")
		.map(|id| Uuid::from_str(id).unwrap())
		.expect("Expected id in path param");

	ctx.state.db.write().unwrap().remove(&id);
	ctx.status(200);

	Ok(ctx)
}
