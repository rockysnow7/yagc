# yagc

yagc (*yagsee*) is yet another Gemini client.

It implements the client standard for the Gemini protocol described at [gemini://geminiprotocol.net/docs/protocol-specification.gmi](gemini://geminiprotocol.net/docs/protocol-specification.gmi).

This is **not** a browser, it is a Rust crate which may be used for implementing a browser or other Gemini client tools.

## Usage

```rust
use yagc::{Client, URL, Request, TlsProtocolVersion};

#[tokio::main]
async fn main() {
    let url = URL::try_from("gemini://geminiprotocol.net/docs/protocol-specification.gmi").unwrap();
    let request = Request(url);

    let client = Client::new();
    let mut connection = client.establish_tls_connection(&request.0).await.unwrap();

    if connection.protocol_version == TlsProtocolVersion::Tls1_3 {
        let response = client.send_request(request, &mut connection).await.unwrap();
        println!("{response:#?}");
    } else {
        println!("TLS 1.3 is required");
    }
}
```

yagc supports:

- Parsing or manually building URLs with the `gemini` or `about` schemes.
- Sending TLS-encrypted requests and receiving responses from Gemini servers.
- Trust-on-first-use (TOFU) certificate verification.

## TODO

- A more secure certificate verification system.
- Maybe don't use an enum for schemes?
