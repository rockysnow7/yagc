# yagc

yagc (*yagsee*) is yet another Gemini client.

It implements the client standard for the Gemini protocol described at [gemini://geminiprotocol.net/docs/protocol-specification.gmi](gemini://geminiprotocol.net/docs/protocol-specification.gmi).

This is **not** a browser, it is a Rust crate which may be used for implementing a browser or other Gemini client tools.

## Usage

```rust
use yagc::{Client, URL, Request};

#[tokio::main]
async fn main() {
    let client = Client::new();

    let url = URL::try_from("gemini://geminiprotocol.net/docs/protocol-specification.gmi").unwrap();
    let request = Request(url);

    let response = client.send_request(request).await;
    println!("{response:#?}");
}
```

yagc supports:

- Parsing or manually building URLs with the `gemini` or `about` schemes.
- Sending TLS-encrypted requests and receiving responses from Gemini servers.
- Trust-on-first-use (TOFU) certificate verification.
- Accepts `text/plain` or `text/gemini` MIME types and `utf-8` or `us-ascii` character sets.

## TODO

- A more secure certificate verification system.
- More schemes.
- More MIME types.
- More character sets.
