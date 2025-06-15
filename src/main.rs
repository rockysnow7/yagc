mod client;
mod url;

use client::{Client, Request};
use url::URLBuilder;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let request = Request(URLBuilder::new()
        .hostname("geminiprotocol.net")
        .path("/docs/protocol-specification.gmi")
        .build());

    let response = client.send_request(request);
    println!("{:#?}", response.await);
}
