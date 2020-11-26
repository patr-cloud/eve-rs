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


<H3> Import `eve-rs` crate </H3>

First step is to bring eve_rs (express port crate) to the scope of the project. so we add the following line

```rust
use eve_rs::{App as DemoApp, Context, DefaultContext, NextHandler, Error, listen, Request, DefaultMiddleware};
```

<H3> middleware </H3>  

Just like in `express` framework, a middleware is basically a function that has access to request and response objects. In `express-port`, we use the same terms, and use middlewares for routing purposes.

for our demo application, we will create a single middleware called `plaintext`, which will print out a plain text on a get request.

a middleware takes in two parameters, `Context` and `NextHandler`. 

`Context` gives you APIs which gives you more information related to the request. Such as getting request body, request status, content type and much more. Since it is a trait (Similar to Interfaces in other programming languages), we get the freedom to define our own context for the application, and on the other hand, the user can use `DefaultContext` provide by express.

`NextHandler` is the next middleware in the sequence. That is, if the current middleware called finishes processing and does not throw an error, and next middleware is passed, then after executing the current middleware, the next middleware is called.

In our example code, we do not have a next middleware to be executed, so we give `_next` as the parameter.

```rust 
async fn plaintext(
    mut context: DefaultContext,
    _next: NextHandler<DefaultContext>
    ) -> Result<DefaultContext, Error<DefaultContext>> {

    let val = "Hello, World!";
    context.body(val);
    
    /**
    Use this if your code uses a next handler.
    let response = next(context).await;
    return response;
    */
    Ok(context)
}
```

<H3> Create App </H3>  

Our next step is to create an express port App. The App struct in express-port gives us a `create()`  function, to create an App. The function takes in Two parameters; `context_generator` and  `state`. 

`context_generator` is a function that is responsible to create a context for our middlewares. It takes in two parameters `Request` and  `state`. State, here could be any configuration we need our app to have. Lets assume that we need our app to have some state, so we will pass the state in the following way.

```rust
pub struct State {
    pub databaseName : String;
}

fn context_generator(request : Request, state : &State) -> YourAppContext {
    let state = state.clone();
    YourAppContext::new(request, state)
}
```

Since, in our example we are using `DefauleContext`, we can create a context without passing in the state.
``` rust
fn default_context_generator(request: Request, _ : &()) -> DefaultContext {     
    // default context has no state as an argument.
	DefaultContext::new(request)
}
```

Once we have a context generator, we can go ahead and create an app.

``` rust
pub fn create_app() -> DemoApp<DefaultContext, DefaultMiddleware<()>, ()>  {
    DemoApp::<DefaultContext, DefaultMiddleware<()>, ()>::create(default_context_generator, ())
}
```
in the above code, since we do not have a state we pass in `()` in create function.

Once our app is created, we add the middlewares in the scope of the app by using `app.use_middleware()` function.

``` rust
 app.use_middleware("/plaintext", &[DefaultMiddleware::new(|context, next| {
        Box::pin(async move {
            plaintext(context, next).await 
        })
    })])
```

Below is how our code will look by combining all the above explained methods.

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
    

    // assign port number.
    let port = 8080;

    log::info!("Listening for connections on 127.0.0.1:{}", port);
    listen(app, ([127, 0, 0, 1], port), None).await;
}

```
Build the project using `cargo build` command.  
Run the project using `cargo run` command.

In the above example we have used  `DefaultContext`  as the  `Context` . This means that express-port gives the freedom to Implement the `Context` in your own way. For the sake of simplicity we have used  `DefaultContext`.

Same goes with `DefaultMiddleware`. here we have used the already implemented middleware.
