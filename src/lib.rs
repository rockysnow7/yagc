//! # yagc
//!
//! yagc (*yagsee*) is yet another Gemini client.
//!
//! It implements the client standard for the Gemini protocol described at [gemini://geminiprotocol.net/docs/protocol-specification.gmi](gemini://geminiprotocol.net/docs/protocol-specification.gmi).

#![warn(missing_docs)]
#![warn(unused_imports)]
#![warn(unused_crate_dependencies)]

mod client;
mod url;

pub use client::{
    Client,
    ClientError,
    request::Request,
    response::{Response, MimeType, MimeTypeType, Charset},
};
pub use url::{URL, URLBuilder, Host, Scheme};
