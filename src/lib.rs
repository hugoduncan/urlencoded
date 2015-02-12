#![feature(core,io)]
#![deny(missing_docs)]

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
use std::error::Error;
use std::fmt::{self, Display};
use std::str::{self, Utf8Error};

use plugin::{Plugin, Pluggable};
use typemap::Key;

pub use UrlDecodingError::*;

/// Plugin for `Request` that extracts URL encoded data from the URL query string.
///
/// Use it like this: `req.get_ref::<UrlEncodedQuery>()`
pub struct UrlEncodedQuery;

/// Plugin for `Request` that extracts URL encoded data from the request body.
///
/// Use it like this: `req.get_ref::<UrlEncodedBody>()`
pub struct UrlEncodedBody;

/// Error type for reporting decoding errors.
#[derive(Clone, Debug, PartialEq)]
pub enum UrlDecodingError{
    /// A UTF-8 encoding issue.
    EncodingError(Utf8Error),
    /// An Empty query string was found.
    EmptyQuery
}

impl Display for UrlDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &EncodingError(err) => write!(f, "Invalid query string: {}", err),
            &EmptyQuery => write!(f, "An emppty query string was found")
        }
    }
}

impl Error for UrlDecodingError {
    fn description(&self) -> &str {
        "Error decoding a UrlEncoded query or body"
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &EncodingError(ref err) => Some(err),
            _ => None
        }
    }
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
    type Error = UrlDecodingError;
    fn eval(req: &mut Request) -> Result<QueryMap, UrlDecodingError> {
        match req.url.query {
            Some(ref query) => create_param_hashmap(&query[]),
            None => Err(UrlDecodingError::EmptyQuery)
        }
    }
}

impl<'a> Plugin<Request<'a>> for UrlEncodedBody {
    type Error = UrlDecodingError;
    fn eval(req: &mut Request) -> Result<QueryMap, UrlDecodingError> {
        str::from_utf8(&req.body.read_to_end().unwrap())
            .or_else(|e| Err(UrlDecodingError::EncodingError(e)))
            .and_then(create_param_hashmap)
    }
}

/// Parse a urlencoded string into an optional HashMap.
fn create_param_hashmap(data: &str) -> Result<QueryMap, UrlDecodingError> {
    match data {
        "" => Err(UrlDecodingError::EmptyQuery),
        _ => Ok(combine_duplicates(form_urlencoded::parse(data.as_bytes())))
    }
}

/// Convert a list of (key, value) pairs into a hashmap with vector values.
fn combine_duplicates(q: Vec<(String, String)>) -> QueryMap {

    let mut deduplicated: QueryMap = HashMap::new();

    for (k, v) in q.into_iter() {
        match deduplicated.entry(k) {
            // Already a Vec here, push onto it
            Entry::Occupied(entry) =>  { entry.into_mut().push(v.clone()); }
            Entry::Vacant(vacant) => { vacant.insert(vec![v]); }
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
