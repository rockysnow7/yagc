use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::opt,
    sequence::preceded,
    IResult,
    Parser,
};

/// The type of a MIME type.
#[derive(Debug, PartialEq)]
pub enum MimeTypeType {
    /// A Gemtext document (`text/gemini`).
    TextGemini,
    /// A plain text document (`text/plain`).
    TextPlain,
}

impl ToString for MimeTypeType {
    fn to_string(&self) -> String {
        match self {
            Self::TextPlain => "text/plain",
            Self::TextGemini => "text/gemini",
        }.to_string()
    }
}

/// A character set.
#[derive(Debug, PartialEq)]
pub enum Charset {
    /// UTF-8.
    Utf8,
    /// US-ASCII.
    UsAscii,
}

impl ToString for Charset {
    fn to_string(&self) -> String {
        match self {
            Self::Utf8 => "utf-8",
            Self::UsAscii => "us-ascii",
        }.to_string()
    }
}

/// A MIME type.
#[derive(Debug, PartialEq)]
pub struct MimeType {
    /// The type of the MIME type.
    pub mime_type: MimeTypeType,
    /// The character set of the MIME type.
    pub charset: Charset,
}

impl ToString for MimeType {
    fn to_string(&self) -> String {
        format!("{};charset={}", self.mime_type.to_string(), self.charset.to_string())
    }
}

impl MimeType {
    /// Create a new `MimeType`.
    pub fn new(mime_type: MimeTypeType, charset: Option<Charset>) -> Self {
        Self { mime_type, charset: charset.unwrap_or(Charset::Utf8) }
    }
}

/// A response to a request.
/// See [gemini://geminiprotocol.net/docs/protocol-specification.gmi](gemini://geminiprotocol.net/docs/protocol-specification.gmi) for more information on what these mean and how they should be handled.
#[derive(Debug, PartialEq)]
pub enum Response {
    /// A request for input from the user.
    Input {
        /// The prompt that should be displayed to the user.
        prompt: String,
    },
    /// A request for sensitive input from the user.
    SensitiveInput {
        /// The prompt that should be displayed to the user.
        prompt: String,
    },
    /// A successful response.
    Success {
        /// The MIME type of the body.
        body_mime_type: MimeType,
        /// The body of the response.
        body: String,
    },
    /// A temporary redirect to a new URL.
    TemporaryRedirect {
        /// The URL to redirect to.
        url: String,
    },
    /// A permanent redirect to a new URL.
    PermanentRedirect {
        /// The URL to redirect to.
        url: String,
    },
    /// A temporary failure.
    TemporaryFailure {
        /// Information about the failure.
        information: String,
    },
    /// The server is currently unavailable.
    ServerUnavailable {
        /// Information about the failure.
        information: String,
    },
    /// A CGI error.
    CGIError {
        /// Information about the failure.
        information: String,
    },
    /// A proxy error.
    ProxyError {
        /// Information about the failure.
        information: String,
    },
    /// A rate limit was enforced, the server is asking the client to slow down.
    SlowDown {
        /// Information about the failure.
        information: String,
    },
    /// A permanent failure.
    PermanentFailure {
        /// Information about the failure.
        information: String,
    },
    /// The requested resource was not found.
    NotFound {
        /// Information about the failure.
        information: String,
    },
    /// The requested resource is no longer available.
    Gone {
        /// Information about the failure.
        information: String,
    },
    /// The proxy request was refused.
    ProxyRequestRefused {
        /// Information about the failure.
        information: String,
    },
    /// The request was invalid.
    BadRequest {
        /// Information about the failure.
        information: String,
    },
    /// A client certificate is required.
    ClientCertificateRequired {
        /// Information about the failure.
        information: String,
    },
    /// The client certificate given was not authorized.
    CertificateNotAuthorized {
        /// Information about the failure.
        information: String,
    },
    /// The client certificate given was not valid.
    CertificateNotValid {
        /// Information about the failure.
        information: String,
    },
}

fn format_response(response_code: u8, response_meta: &str, body: &str) -> String {
    format!("{response_code} {response_meta}\r\n{body}")
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Self::Input { prompt }                          => format_response( 10, prompt,                                 ""      ),
            Self::SensitiveInput { prompt }                 => format_response( 11, prompt,                                 ""      ),
            Self::Success { body_mime_type, body }          => format_response( 20, body_mime_type.to_string().as_str(),    body    ),
            Self::TemporaryRedirect { url }                 => format_response( 30, url,                                    ""      ),
            Self::PermanentRedirect { url }                 => format_response( 31, url,                                    ""      ),
            Self::TemporaryFailure { information }          => format_response( 40, information,                            ""      ),
            Self::ServerUnavailable { information }         => format_response( 41, information,                            ""      ),
            Self::CGIError { information }                  => format_response( 42, information,                            ""      ),
            Self::ProxyError { information }                => format_response( 43, information,                            ""      ),
            Self::SlowDown { information }                  => format_response( 44, information,                            ""      ),
            Self::PermanentFailure { information }          => format_response( 50, information,                            ""      ),
            Self::NotFound { information }                  => format_response( 51, information,                            ""      ),
            Self::Gone { information }                      => format_response( 52, information,                            ""      ),
            Self::ProxyRequestRefused { information }       => format_response( 53, information,                            ""      ),
            Self::BadRequest { information }                => format_response( 59, information,                            ""      ),
            Self::ClientCertificateRequired { information } => format_response( 60, information,                            ""      ),
            Self::CertificateNotAuthorized { information }  => format_response( 61, information,                            ""      ),
            Self::CertificateNotValid { information }       => format_response( 62, information,                            ""      ),
        }
    }
}

// this is just a collection of parsers for the different response types
impl Response {
    fn input_expected(input: &str) -> IResult<&str, Self> {
        let (input, response_code) = alt((tag("10"), tag("11"))).parse(input)?;
        let (input, _) = tag(" ").parse(input)?;
        let (input, prompt) = take_until("\r\n").parse(input)?;
        let (input, _) = tag("\r\n").parse(input)?;

        let response = match response_code {
            "10" => Self::Input { prompt: prompt.to_string() },
            "11" => Self::SensitiveInput { prompt: prompt.to_string() },
            _ => unreachable!(),
        };

        Ok((input, response))
    }

    fn charset(input: &str) -> IResult<&str, Charset> {
        let (input, charset) = alt((tag("utf-8"), tag("us-ascii"))).parse(input)?;

        let charset = match charset {
            "utf-8" => Charset::Utf8,
            "us-ascii" => Charset::UsAscii,
            _ => unreachable!(),
        };

        Ok((input, charset))
    }

    fn mime_type(input: &str) -> IResult<&str, MimeType> {
        let (input, mime_type) = alt((tag("text/plain"), tag("text/gemini"))).parse(input)?;
        let (input, charset) = opt(preceded(
            tag(";charset="),
            Self::charset,
        )).parse(input)?;

        let mime_type = match mime_type {
            "text/plain" => MimeType::new(MimeTypeType::TextPlain, charset),
            "text/gemini" => MimeType::new(MimeTypeType::TextGemini, charset),
            _ => unreachable!(),
        };

        Ok((input, mime_type))
    }

    fn success(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("20 ").parse(input)?;
        let (input, body_mime_type) = Self::mime_type(input)?;
        let (input, _) = tag("\r\n").parse(input)?;

        let response = Self::Success { body_mime_type, body: input.to_string() };

        Ok(("", response))
    }

    fn temporary_redirect(input: &str) -> IResult<&str, Self> {
        let (input, response_code) = alt((tag("30"), tag("31"))).parse(input)?;
        let (input, _) = tag(" ").parse(input)?;
        let (input, url) = take_until("\r\n").parse(input)?;
        let url = url.to_string();
        let (input, _) = tag("\r\n").parse(input)?;

        let response = match response_code {
            "30" => Self::TemporaryRedirect { url },
            "31" => Self::PermanentRedirect { url },
            _ => unreachable!(),
        };

        Ok((input, response))
    }

    fn temporary_failure(input: &str) -> IResult<&str, Self> {
        let (input, response_code) = alt((tag("40"), tag("41"), tag("42"), tag("43"), tag("44"))).parse(input)?;
        let (input, _) = tag(" ").parse(input)?;
        let (input, information) = take_until("\r\n").parse(input)?;
        let information = information.to_string();
        let (input, _) = tag("\r\n").parse(input)?;

        let response = match response_code {
            "40" => Self::TemporaryFailure { information },
            "41" => Self::ServerUnavailable { information },
            "42" => Self::CGIError { information },
            "43" => Self::ProxyError { information },
            "44" => Self::SlowDown { information },
            _ => unreachable!(),
        };

        Ok((input, response))
    }

    fn permanent_failure(input: &str) -> IResult<&str, Self> {
        let (input, response_code) = alt((tag("50"), tag("51"), tag("52"), tag("53"), tag("59"))).parse(input)?;
        let (input, _) = tag(" ").parse(input)?;
        let (input, information) = take_until("\r\n").parse(input)?;
        let information = information.to_string();
        let (input, _) = tag("\r\n").parse(input)?;

        let response = match response_code {
            "50" => Self::PermanentFailure { information },
            "51" => Self::NotFound { information },
            "52" => Self::Gone { information },
            "53" => Self::ProxyRequestRefused { information },
            "59" => Self::BadRequest { information },
            _ => unreachable!(),
        };

        Ok((input, response))
    }

    fn client_certificate_required(input: &str) -> IResult<&str, Self> {
        let (input, response_code) = alt((tag("60"), tag("61"), tag("62"))).parse(input)?;
        let (input, _) = tag(" ").parse(input)?;
        let (input, information) = take_until("\r\n").parse(input)?;
        let information = information.to_string();
        let (input, _) = tag("\r\n").parse(input)?;

        let response = match response_code {
            "60" => Self::ClientCertificateRequired { information },
            "61" => Self::CertificateNotAuthorized { information },
            "62" => Self::CertificateNotValid { information },
            _ => unreachable!(),
        };

        Ok((input, response))
    }

    fn from_str(input: &str) -> IResult<&str, Self> {
        alt((
            Self::input_expected,
            Self::success,
            Self::temporary_redirect,
            Self::temporary_failure,
            Self::permanent_failure,
            Self::client_certificate_required,
        )).parse(input)
    }
}

impl TryFrom<&str> for Response {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let (input, response) = Self::from_str(input).map_err(|e| e.to_string())?;

        if !input.is_empty() {
            Err(format!("Unexpected input: {input}"))
        } else {
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input() {
        let response = Response::try_from("10 What is the capital of France?\r\n");
        assert_eq!(response, Ok(Response::Input { prompt: "What is the capital of France?".to_string() }));
    }

    #[test]
    fn sensitive_input() {
        let response = Response::try_from("11 What is the capital of France?\r\n");
        assert_eq!(response, Ok(Response::SensitiveInput { prompt: "What is the capital of France?".to_string() }));
    }

    #[test]
    fn success_without_charset() {
        let response = Response::try_from("20 text/plain\r\nHello, world!");
        assert_eq!(response, Ok(Response::Success {
            body_mime_type: MimeType::new(MimeTypeType::TextPlain, Some(Charset::Utf8)),
            body: "Hello, world!".to_string(),
        }));
    }

    #[test]
    fn success_with_charset() {
        let response = Response::try_from("20 text/plain;charset=us-ascii\r\nHello, world!");
        assert_eq!(response, Ok(Response::Success {
            body_mime_type: MimeType::new(MimeTypeType::TextPlain, Some(Charset::UsAscii)),
            body: "Hello, world!".to_string(),
        }));
    }

    #[test]
    fn temporary_redirect() {
        let response = Response::try_from("30 https://example.com\r\n");
        assert_eq!(response, Ok(Response::TemporaryRedirect { url: "https://example.com".to_string() }));
    }

    #[test]
    fn permanent_redirect() {
        let response = Response::try_from("31 https://example.com\r\n");
        assert_eq!(response, Ok(Response::PermanentRedirect { url: "https://example.com".to_string() }));
    }

    #[test]
    fn temporary_failure() {
        let response = Response::try_from("40 The server is temporarily unable to service your request due to maintenance downtime or capacity problems. Please try again later.\r\n");
        assert_eq!(response, Ok(Response::TemporaryFailure { information: "The server is temporarily unable to service your request due to maintenance downtime or capacity problems. Please try again later.".to_string() }));
    }

    #[test]
    fn server_unavailable() {
        let response = Response::try_from("41 The server is currently unavailable. Please try again later.\r\n");
        assert_eq!(response, Ok(Response::ServerUnavailable { information: "The server is currently unavailable. Please try again later.".to_string() }));
    }

    #[test]
    fn cgi_error() {
        let response = Response::try_from("42 meow\r\n");
        assert_eq!(response, Ok(Response::CGIError { information: "meow".to_string() }));
    }

    #[test]
    fn proxy_error() {
        let response = Response::try_from("43 meow\r\n");
        assert_eq!(response, Ok(Response::ProxyError { information: "meow".to_string() }));
    }

    #[test]
    fn slow_down() {
        let response = Response::try_from("44 meow\r\n");
        assert_eq!(response, Ok(Response::SlowDown { information: "meow".to_string() }));
    }

    #[test]
    fn permanent_failure() {
        let response = Response::try_from("50 meow\r\n");
        assert_eq!(response, Ok(Response::PermanentFailure { information: "meow".to_string() }));
    }

    #[test]
    fn not_found() {
        let response = Response::try_from("51 meow\r\n");
        assert_eq!(response, Ok(Response::NotFound { information: "meow".to_string() }));
    }

    #[test]
    fn gone() {
        let response = Response::try_from("52 meow\r\n");
        assert_eq!(response, Ok(Response::Gone { information: "meow".to_string() }));
    }

    #[test]
    fn proxy_request_refused() {
        let response = Response::try_from("53 meow\r\n");
        assert_eq!(response, Ok(Response::ProxyRequestRefused { information: "meow".to_string() }));
    }

    #[test]
    fn bad_request() {
        let response = Response::try_from("59 meow\r\n");
        assert_eq!(response, Ok(Response::BadRequest { information: "meow".to_string() }));
    }

    #[test]
    fn client_certificate_required() {
        let response = Response::try_from("60 meow\r\n");
        assert_eq!(response, Ok(Response::ClientCertificateRequired { information: "meow".to_string() }));
    }

    #[test]
    fn certificate_not_authorized() {
        let response = Response::try_from("61 meow\r\n");
        assert_eq!(response, Ok(Response::CertificateNotAuthorized { information: "meow".to_string() }));
    }

    #[test]
    fn certificate_not_valid() {
        let response = Response::try_from("62 meow\r\n");
        assert_eq!(response, Ok(Response::CertificateNotValid { information: "meow".to_string() }));
    }

    #[test]
    fn invalid_response() {
        let response = Response::try_from("70 meow\r\n");
        assert!(response.is_err());
    }
}
