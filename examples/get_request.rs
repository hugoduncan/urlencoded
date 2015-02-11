#![feature(io)]

//! This example shows how to use urlencoded to parse GET request parameters.
extern crate iron;
extern crate urlencoded;

use std::old_io::net::ip::Ipv4Addr;

use iron::{Iron, Request, Response, IronResult, Plugin, Set, status};
use urlencoded::UrlEncodedQuery;

fn log_params(req: &mut Request) -> IronResult<Response> {
    // Extract the decoded data as hashmap, using the UrlEncodedQuery plugin.
    match req.get_ref::<UrlEncodedQuery>() {
        Ok(hashmap) => println!("Parsed GET request query string:\n {:?}", hashmap),
        Err(x) => println!("Error, no query string found: {:?}", x)
    };

    Ok(Response::new().set((status::Ok,"Hello!")))
}

// Test out the server with `curl -i "http://localhost:3000/?name=franklin&name=trevor"`
fn main() {
    Iron::new(log_params).listen((Ipv4Addr(127, 0, 0, 1), 3000)).unwrap();
}
