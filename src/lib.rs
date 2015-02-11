#![feature(core,io)]

//! URL Encoded Plugin for Iron.
//!
//! Parses "url encoded" data from client requests.
//! Capable of parsing both URL query strings and POST request bodies.

extern crate iron;
extern crate url;

extern crate plugin;
extern crate typemap;

use iron::Request;

use url::form_urlencoded;
use std::collections::hash_map::{Entry, HashMap};
use std::str::{self, Utf8Error};

use plugin::{Plugin, Pluggable};
use typemap::Key;

/// Plugin for `Request` that extracts URL encoded data from the URL query string.
///
/// Use it like this: `req.get_ref::<UrlEncodedQuery>()`
pub struct UrlEncodedQuery;

/// Plugin for `Request` that extracts URL encoded data from the request body.
///
/// Use it like this: `req.get_ref::<UrlEncodedBody>()`
pub struct UrlEncodedBody;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorType{
    EncodingError(Utf8Error),
    EmptyQuery
}

/// Hashmap mapping strings to vectors of strings.
pub type QueryMap = HashMap<String, Vec<String>>;

impl Pluggable for UrlEncodedQuery {}

impl Pluggable for UrlEncodedBody {}

impl Key for UrlEncodedQuery {
    type Value = QueryMap;
}

impl Key for UrlEncodedBody {
    type Value = QueryMap;
}

impl<'a> Plugin<Request<'a>> for UrlEncodedQuery {
    type Error = ErrorType;
    fn eval(req: &mut Request) -> Result<QueryMap, ErrorType> {
        match req.url.query {
            Some(ref query) => create_param_hashmap(&query[]),
            None => Err(ErrorType::EmptyQuery)
        }
    }
}

impl<'a> Plugin<Request<'a>> for UrlEncodedBody {
    type Error = ErrorType;
    fn eval(req: &mut Request) -> Result<QueryMap, ErrorType> {
        str::from_utf8(&req.body.read_to_end().unwrap())
            .or_else(|e| Err(ErrorType::EncodingError(e)))
            .and_then(create_param_hashmap)
    }
}

/// Parse a urlencoded string into an optional HashMap.
fn create_param_hashmap(data: &str) -> Result<QueryMap, ErrorType> {
    match data {
        "" => Err(ErrorType::EmptyQuery),
        _ => Ok(combine_duplicates(form_urlencoded::parse(data.as_bytes())))
    }
}

/// Convert a list of (key, value) pairs into a hashmap with vector values.
fn combine_duplicates(q: Vec<(String, String)>) -> QueryMap {

    let mut deduplicated: QueryMap = HashMap::new();

    for (k, v) in q.into_iter() {
        let is_new = match deduplicated.entry(k.clone()) {
            // Already a Vec here, push onto it
            Entry::Occupied(entry) => { entry.into_mut().push(v.clone()); false},
            Entry::Vacant(_) => true
        };
        if is_new {
            // No value, create a one-element Vec.
            deduplicated.insert(k.clone(), vec![v]);
        };
    }

    deduplicated
}

#[test]
fn test_combine_duplicates() {
    let my_vec = vec!(("band".to_string(), "arctic monkeys".to_string()),
                      ("band".to_string(), "temper trap".to_string()),
                      ("color".to_string(),"green".to_string()));
    let answer = combine_duplicates(my_vec);
    let mut control = HashMap::new();
    control.insert("band".to_string(),
                   vec!("arctic monkeys".to_string(), "temper trap".to_string()));
    control.insert("color".to_string(), vec!("green".to_string()));
    assert_eq!(answer, control);
}
