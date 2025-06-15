use crate::url::URL;

#[derive(Debug)]
pub struct Request(pub URL);

impl ToString for Request {
    fn to_string(&self) -> String {
        format!("{}\r\n", self.0.to_string())
    }
}

impl Request {
    pub fn is_valid_length(&self) -> bool {
        self.0.to_string().bytes().count() <= 1024
    }
}
