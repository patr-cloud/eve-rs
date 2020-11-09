# Express-Port

## Express port is a rust http framework which is inspired from Express Node JS framework.



### Example

Let's create a demo application using Express-port, using an intuative approach to get a better understanding of the framework.

Create a cargo project, here we have named it demo, and in the `cargo.toml` file, add the following dependencies under the depedency section.

``` rust
[dependencies]
eve-rs = '0.1.0'
log = '0.4.8'
async-std = "=1.6.2"
```

here, `eve-rs` is the crate name for express port.

after updating toml file, we then enter the following code in the `main.rs` file

``` rust

// Bring express port modules to the program scope.
use eve_rs::{App as DemoApp, Context, DefaultContext, NextHandler, Error, listen, Request, DefaultMiddleware};

// middleware
async fn plaintext(
    mut context: DefaultContext,
     _: NextHandler<DefaultContext>
    ) -> Result<DefaultContext, Error<DefaultContext>> {

    let val = "Hello, World!";
    context.body(val);
    
    Ok(context)
}

// function to create an express port app.
pub fn create_app() -> DemoApp<DefaultContext, DefaultMiddleware<()>, ()>  {
    DemoApp::<DefaultContext, DefaultMiddleware<()>, ()>::create(default_context_generator, ())
}


// context generator.
fn default_context_generator(request: Request, _ : &()) -> DefaultContext {     
    // default context has no state as an argument.
	DefaultContext::new(request)
}


#[async_std::main]
async fn main() {
    println!("Starting server...");

    // call function to create default app.
    let mut app = create_app();
    
    // add middleware to the stack.
    app.use_middleware("/plaintext", &[DefaultMiddleware::new(|context, next| {
        Box::pin(async move {
            plaintext(context, next).await 
        })
    })]);
    

    
    // define an endpoint and the function/middleware it will execute at that endpoint.
    app.get("/plaintext", &[DefaultMiddleware::new(|context, next| {
        Box::pin(async move {
            println!("Route: {}", context.get_full_url());
            plaintext(context, next).await 
        })
    })]);

    // assuign port number.
    let port = 8080;

    log::info!("Listening for connections on 127.0.0.1:{}", port);
    listen(app, ([127, 0, 0, 1], port), None).await;
}

```
build the project using `cargo build` command.  
run the project using `cargo run` command.

In the above example we have used  `DefaultContext`  as the  `Context` . This means that express-port gives the freedom to Implement the `Context` in your own way. for the sake of simplicity we have used  `DefaultContext`.

Same goes with `DefaultMiddleware`. here we have used the already implemented middleware.