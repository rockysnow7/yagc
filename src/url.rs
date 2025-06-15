const DEFAULT_PORT: u16 = 1965;
const DEFAULT_SCHEME: &str = "gemini";
const DEFAULT_PATH: &str = "/";

#[derive(Debug)]
pub struct Host {
    pub name: String,
    pub port: u16,
}

impl ToString for Host {
    fn to_string(&self) -> String {
        format!("//{}:{}", self.name, self.port)
    }
}

#[derive(Debug)]
pub struct URL {
    scheme: String,
    pub host: Option<Host>,
    path: String,
    query: Option<String>,
}

impl ToString for URL {
    fn to_string(&self) -> String {
        let mut uri = format!("{}:", self.scheme);

        if let Some(host) = &self.host {
            uri.push_str(&host.to_string());
        }

        if self.path.is_empty() {
            uri.push_str("/");
        } else {
            uri.push_str(&self.path);
        }

        if let Some(query) = &self.query {
            uri.push_str("?");
            uri.push_str(&query);
        }

        uri
    }
}

pub struct URLBuilder<'a> {
    scheme: Option<&'a str>,
    hostname: Option<&'a str>,
    port: Option<u16>,
    path: Option<&'a str>,
    query: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> URLBuilder<'a> {
    pub fn new() -> Self {
        Self {
            scheme: None,
            hostname: None,
            port: None,
            path: None,
            query: None,
        }
    }

    pub fn scheme(mut self, scheme: &'a str) -> Self {
        self.scheme = Some(scheme);
        self
    }

    pub fn hostname(mut self, hostname: &'a str) -> Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    fn host(&self) -> Option<Host> {
        if let Some(hostname) = &self.hostname {
            Some(Host {
                name: hostname.to_string(),
                port: self.port.unwrap_or(DEFAULT_PORT),
            })
        } else {
            None
        }
    }

    pub fn build(&self) -> URL {
        let scheme = self.scheme.unwrap_or(DEFAULT_SCHEME);
        let host = self.host();
        let path = self.path.unwrap_or(DEFAULT_PATH);
        let query = self.query.map(|query| query.to_string());

        URL {
            scheme: scheme.to_string(),
            host,
            path: path.to_string(),
            query,
        }
    }
}
