#[macro_use]
extern crate lando;

use lando::{IntoResponse, LambdaContext, Request, Result};

#[lando]
pub fn example(_: Request, _: LambdaContext) -> Result<impl IntoResponse> {
    Ok("👋  well hello there. What have we here?")
}
