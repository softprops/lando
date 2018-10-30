#[macro_use] extern crate lando;

use lando::{LambdaContext, Request, Response, Result};

#[lando]
pub fn example(_: Request, _: LambdaContext) -> Result {
    Ok(Response::new("👋  well hello there. What have we here?".into()))
}