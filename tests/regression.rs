#![allow(clippy::elidable_lifetime_names, clippy::needless_lifetimes)]

extern crate laxjson as serde_json;

mod regression {
    automod::dir!("tests/regression");
}
