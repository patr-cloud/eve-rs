use express;

async fn main() {
	let app = express::new();

	app.use_middleware([middleware]);
	app.use_router(express::router());

	express::listen(app, [127, 0, 0, 1], 3000).await;
}

async fn middleware(context: Context, next: NextHandler<Context>) -> Result<Context, Error> {
	
}