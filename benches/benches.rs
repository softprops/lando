// run cargo +nightly bench
#![feature(test)]

extern crate lando;
extern crate serde_json;
extern crate test;

use lando::GatewayRequest;
use serde_json::Value;

#[bench]
fn gateway_conversion(b: &mut test::Bencher) {
    let event = serde_json::from_str::<Value>(include_str!("request.json")).unwrap();
    b.iter(|| serde_json::from_value::<GatewayRequest>(event.clone()).unwrap());
}
