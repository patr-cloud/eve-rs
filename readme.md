# Express-Port

## Express port is a rust http framework which is inspired from Express Node JS framework.



### Example

We will create a demo application using Express-port, using an intuative approach to get a better understanding of the framework.


file name : ```demo.rs```
``` rust

//Eve-rs is the package that gives access to Express port files.
use eve_rs::{App, Context::DefaultContext as Context, Error, NextHandler};

// middleware
async fn plaintext(mut context: Context, _: NextHandler<Context>) -> Result<Context, Error<Context>> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

fn main() {
    println!("Starting server...");

    // get default app
    let mut app = App::<Request, Context>::default(); 
    
    // define an endpoint and the function/middleware it will execute at that endpoint.
    app.get("/plaintext", &[plaintext]));

    let port = 8081;

    log::info!("Listening for connections on 127.0.0.1:{}", port);
    listen(app, ([127, 0, 0, 1], port), None).await;
}

```
run the example using ```cargo run``` command.

In the above example we have used  `DefaultContext`  as the  `Context` . This means that express-port gives the freedom to Implement the `Context` in your own way. for the sake of simplicity we have used  `DefaultContext`.