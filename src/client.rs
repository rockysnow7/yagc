pub mod request;
pub mod response;
mod tofu;

use crate::url::URL;
use request::Request;
use response::Response;
use std::sync::Arc;
use tofu::{TofuStore, TofuVerifier};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{client::TlsStream, TlsConnector};
use rustls::pki_types::ServerName;

/// An error that can occur when the client tries to do something.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ClientError {
    /// The request is too long (more than 1024 bytes).
    RequestTooLong(String),
    /// The host address could not be resolved.
    FailedToResolveHostAddress(String),
    /// The connection to the host could not be established.
    FailedToConnectToHost(String),
    /// A response from the host was received but could not be parsed.
    FailedToReadResponse(String),
}

/// A client for the Gemini protocol.
pub struct Client {
    tofu_store: TofuStore,
}

impl Client {
    /// Create a new client with a TOFU store loaded from the default path.
    pub fn new() -> Self {
        Self { tofu_store: TofuStore::new("known_hosts.json".to_string()).unwrap() }
    }

    /// Establish a TLS connection with a host.
    async fn establish_tls_connection(&self, url: &URL) -> Result<TlsStream<TcpStream>, ClientError> {
        // get the hostname and port from the url
        let (hostname, port) = if let Some(host) = &url.host {
            (host.name.clone(), host.port)
        } else {
            return Err(ClientError::FailedToResolveHostAddress("URL must contain a host".to_string()));
        };

        // create a new tofu verifier
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(TofuVerifier::new(self.tofu_store.clone())))
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));

        // connect to the host
        let tcp_stream = TcpStream::connect((hostname.clone(), port))
            .await
            .map_err(|e| ClientError::FailedToConnectToHost(e.to_string()))?;

        // server name indication
        let domain = ServerName::try_from(hostname)
            .map_err(|e| ClientError::FailedToConnectToHost(e.to_string()))?;

        // establish the tls connection
        let tls_stream = connector.connect(domain, tcp_stream)
            .await
            .map_err(|e| ClientError::FailedToConnectToHost(e.to_string()))?;

        Ok(tls_stream)
    }

    /// Send a request to the host and return the response/error.
    pub async fn send_request(&self, request: Request) -> Result<Response, ClientError> {
        if !request.is_valid_length() {
            let length = request.0.to_string().as_bytes().len();
            return Err(ClientError::RequestTooLong(format!("Request is too long: {length} bytes")));
        }

        let mut tls_stream = self.establish_tls_connection(&request.0).await?;

        if let Err(_) = tls_stream.write_all(request.to_string().as_bytes()).await {
            return Err(ClientError::FailedToConnectToHost(request.0.host.as_ref().unwrap().name.clone()));
        }

        let mut buffer = Vec::new();
        tls_stream.read_to_end(&mut buffer)
            .await
            .map_err(|_| ClientError::FailedToReadResponse("Failed to read response".to_string()))?;

        Response::try_from(String::from_utf8_lossy(&buffer).as_ref()).map_err(|e| ClientError::FailedToReadResponse(e))
    }
}
