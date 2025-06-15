use crate::url::URL;

/// A request to a given URL.
#[derive(Debug)]
pub struct Request(pub URL);

impl ToString for Request {
    fn to_string(&self) -> String {
        format!("{}\r\n", self.0.to_string())
    }
}

impl Request {
    /// Check if the request is valid (less than or equal to 1024 bytes).
    pub fn is_valid_length(&self) -> bool {
        self.0.to_string().bytes().count() <= 1024
    }
}
