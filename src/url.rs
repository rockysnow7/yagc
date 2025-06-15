use nom::{
    branch::alt, bytes::complete::{tag, take_while}, character::digit1, combinator::opt, multi::{many0, many1}, sequence::{preceded, terminated}, IResult, Parser
};

const DEFAULT_PORT: u16 = 1965;
const DEFAULT_PATH: &str = "/";
const DEFAULT_SCHEME: Scheme = Scheme::Gemini;

/// The scheme part of a URL.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Scheme {
    Gemini,
    About,
}

impl ToString for Scheme {
    fn to_string(&self) -> String {
        match self {
            Scheme::Gemini => "gemini",
            Scheme::About => "about",
        }.to_string()
    }
}

/// The host part of a URL.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Clone)]
pub struct Host {
    pub name: String,
    pub port: u16,
}

impl ToString for Host {
    fn to_string(&self) -> String {
        format!("{}:{}", self.name, self.port)
    }
}

/// A URL to a Gemini resource.
#[derive(Debug, PartialEq)]
pub struct URL {
    /// The scheme part of the URL.
    pub scheme: Scheme,
    /// The host part of the URL.
    pub host: Option<Host>,
    /// The path part of the URL.
    pub path: String,
    /// The query part of the URL.
    pub query: Option<String>,
}

impl ToString for URL {
    fn to_string(&self) -> String {
        let mut uri = format!("{}:", self.scheme.to_string());

        if let Some(host) = &self.host {
            uri.push_str("//");
            uri.push_str(&host.to_string());
        }

        if !self.path.starts_with('/') {
            uri.push_str("/");
        }
        uri.push_str(&self.path);

        if let Some(query) = &self.query {
            uri.push_str("?");
            uri.push_str(&query);
        }

        uri
    }
}

// (kinda jank but it works)
// <url> := <scheme> ":" ( "//" <hostname> ( ":" <port> )? )? <path> ( "?" <query> )?
// <hostname> := <url char>+ ( "." <url char>+ )+
// <path> := ( "/" <url char>+ )*
impl URL {
    fn scheme(input: &str) -> IResult<&str, Scheme> {
        terminated(
            alt((
                tag("gemini"),
                tag("about"),
            )),
            tag(":"),
        )
        .parse(input)
        .map(|(input, scheme)| {
            let scheme = match scheme {
                "gemini" => Scheme::Gemini,
                "about" => Scheme::About,
                _ => unreachable!(),
            };

            (input, scheme)
        })
    }

    fn hostname(input: &str) -> IResult<&str, String> {
        (
            take_while(|c: char| c != '.' && c != '/' && c != ':'),
            many1(preceded(
                tag("."),
                take_while(|c: char| c != '.' && c != '/' && c != ':'),
            )),
        )
        .parse(input)
        .map(|(input, (part, parts))| {
            let mut hostname = part.to_string();
            for part in parts {
                hostname.push('.');
                hostname.push_str(part);
            }

            (input, hostname)
        })
    }
    
    fn port(input: &str) -> IResult<&str, u16> {
        digit1()
            .parse(input)
            .map(|(input, port)| {
                let port = port.parse::<u16>().unwrap();

                (input, port)
            })
    }

    fn host(input: &str) -> IResult<&str, Host> {
        preceded(
            tag("//"),
            (
                Self::hostname,
                opt(preceded(
                    tag(":"),
                    Self::port,
                )),
            )
        )
        .parse(input)
        .map(|(input, (hostname, port))| {
            let port = port.unwrap_or(DEFAULT_PORT);

            (input, Host { name: hostname, port })
        })
    }

    fn query(input: &str) -> IResult<&str, String> {
        preceded(
            tag("?"),
            take_while(|_| true),
        )
        .parse(input)
        .map(|(input, query)| (input, query.to_string()))
    }

    fn path(input: &str) -> IResult<&str, String> {
        (
            take_while(|c: char| c != '/' && c != '?' && c != ':'),
            many0(preceded(
                tag("/"),
                take_while(|c: char| c != '/' && c != '?' && c != ':'),
            )),
        )
        .parse(input)
        .map(|(input, (part, parts))| {
            let mut path = part.to_string();
            for part in parts {
                path.push('/');
                path.push_str(part);
            }

            (input, path)
        })
    }

    fn url(input: &str) -> IResult<&str, Self> {
        (
            opt(Self::scheme),
            opt(Self::host),
            Self::path,
            opt(Self::query),
        )
        .parse(input)
        .map(|(input, (scheme, host, path, query))| {
            let mut url_builder = URLBuilder::new();

            if let Some(scheme) = scheme {
                url_builder = url_builder.scheme(scheme);
            }

            // If we have no scheme and no host, but the path looks like a hostname,
            // treat it as a host instead
            if scheme.is_none() && host.is_none() && !path.is_empty() && !path.starts_with('/') {
                // Split the path into hostname and path parts
                let parts: Vec<&str> = path.splitn(2, '/').collect();
                let hostname = parts[0];
                let path_part = if parts.len() > 1 {
                    format!("/{}", parts[1])
                } else {
                    "/".to_string()
                };

                let host = Host {
                    name: hostname.to_string(),
                    port: DEFAULT_PORT,
                };
                url_builder = url_builder.host(host);
                url_builder = url_builder.path(path_part);
            } else {
                if let Some(host) = host {
                    url_builder = url_builder.host(host);
                }

                if !path.is_empty() {
                    url_builder = url_builder.path(path);
                }
            }

            if let Some(query) = query {
                url_builder = url_builder.query(query);
            }

            let url = url_builder.build();

            (input, url)
        })
    }
}

impl TryFrom<&str> for URL {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (input, url) = Self::url(value).map_err(|e| e.to_string())?;

        if !input.is_empty() {
            Err(format!("Invalid URL: {value}"))
        } else {
            Ok(url)
        }
    }
}

/// A builder for `URL`s.
pub struct URLBuilder {
    scheme: Scheme,
    host: Option<Host>,
    path: Option<String>,
    query: Option<String>,
}

#[allow(dead_code)]
impl URLBuilder {
    /// Create a new `URLBuilder`.
    pub fn new() -> Self {
        Self {
            scheme: DEFAULT_SCHEME,
            host: None,
            path: None,
            query: None,
        }
    }

    /// Set the scheme of the URL.
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = scheme;
        self
    }

    /// Set the host of the URL.
    pub fn host(mut self, host: Host) -> Self {
        self.host = Some(host);
        self
    }

    /// Set the path of the URL.
    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Set the query of the URL.
    pub fn query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    /// Build the URL.
    pub fn build(&self) -> URL {
        let path = self.path.as_deref().unwrap_or(DEFAULT_PATH);
        let query = self.query.as_ref().map(|query| query.to_string());

        URL {
            scheme: self.scheme,
            host: self.host.clone(),
            path: path.to_string(),
            query,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_url() {
        let url = URL::try_from("about:meow");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::About,
            host: None,
            path: "meow".to_string(),
            query: None,
        }));
    }

    #[test]
    fn gemini_url_with_host() {
        let url = URL::try_from("gemini://example.com");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::Gemini,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/".to_string(),
            query: None,
        }));
    }

    #[test]
    fn gemini_url_with_path() {
        let url = URL::try_from("gemini://example.com/path");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::Gemini,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/path".to_string(),
            query: None,
        }));
    }

    #[test]
    fn gemini_url_with_query() {
        let url = URL::try_from("gemini://example.com/path?query");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::Gemini,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/path".to_string(),
            query: Some("query".to_string()),
        }));
    }

    #[test]
    fn gemini_url_with_file_path() {
        let url = URL::try_from("gemini://example.com/path/to/file.txt");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::Gemini,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/path/to/file.txt".to_string(),
            query: None,
        }));
    }

    #[test]
    fn url_with_no_scheme() {
        let url = URL::try_from("example.com");

        assert_eq!(url, Ok(URL {
            scheme: DEFAULT_SCHEME,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/".to_string(),
            query: None,
        }));
    }

    #[test]
    fn url_with_path_and_no_scheme() {
        let url = URL::try_from("example.com/path/to/file.txt");
        eprintln!("{url:?}");

        assert_eq!(url, Ok(URL {
            scheme: Scheme::Gemini,
            host: Some(Host {
                name: "example.com".to_string(),
                port: DEFAULT_PORT,
            }),
            path: "/path/to/file.txt".to_string(),
            query: None,
        }));
    }

    #[test]
    fn invalid_scheme() {
        let url = URL::try_from("nooo://a.com");
        eprintln!("{url:?}");

        assert!(url.is_err());
    }
}
