use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, RwLock},
};

use eve_rs::{Context, Error, Middleware, NextHandler, Request, Response};
use futures::Future;
use serde::Serialize;
use uuid::Uuid;

// Middleware used by Eve
#[derive(Debug, Clone)]
pub struct Middler {
    handler: MiddlewareHandler,
}

type MiddlewareHandler =
    fn(
        Ctx,
        NextHandler<Ctx, ()>,
    ) -> Pin<Box<dyn Future<Output = Result<Ctx, Error<()>>> + Send>>;

impl Middler {
    pub fn new(handler: MiddlewareHandler) -> Self {
        Middler { handler }
    }
}

#[async_trait::async_trait]
impl Middleware<Ctx, ()> for Middler {
    async fn run_middleware(
        &self,
        context: Ctx,
        next: NextHandler<Ctx, ()>,
    ) -> Result<Ctx, Error<()>> {
        (self.handler)(context, next).await
    }
}

// Context used by every middleware
#[derive(Debug)]
pub struct Ctx {
    // basic data
    req: Request,
    res: Response,

    // buffering streaming data
    buffered_request_body: Option<Vec<u8>>,

    // additional data
    pub state: AppState,
}

impl Ctx {
    pub fn new(req: Request, state: &AppState) -> Self {
        Ctx {
            req,
            res: Default::default(),
            state: state.clone(),
            buffered_request_body: None,
        }
    }
}

#[async_trait::async_trait]
impl Context for Ctx {
    fn get_request(&self) -> &Request {
        &self.req
    }

    fn get_request_mut(&mut self) -> &mut Request {
        &mut self.req
    }

    fn get_response(&self) -> &Response {
        &self.res
    }

    fn take_response(self) -> Response {
        self.res
    }

    fn get_response_mut(&mut self) -> &mut Response {
        &mut self.res
    }

    type ResBodyBuffer = Vec<u8>;
    async fn get_buffered_request_body(&mut self) -> &Self::ResBodyBuffer {
        match self.buffered_request_body {
            Some(ref body) => body,
            None => {
                let body = hyper::body::to_bytes(self.req.take_body())
                    .await
                    .unwrap()
                    .to_vec();
                self.buffered_request_body.get_or_insert(body)
            }
        }
    }
}

// AppState for sharing and persistance
#[derive(Debug, Default, Clone)]
pub struct AppState {
    // Since state is cloned for every request,
    // size of struct fields should be small
    pub db: Db,
}

type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Clone, Serialize)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
}
