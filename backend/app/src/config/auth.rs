use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    allow_list: Vec<String>,
    userless: bool,
}

impl AuthConfig {
    pub fn allow_list(&self) -> &Vec<String> {
        return &self.allow_list;
    }

    pub fn userless(&self) -> bool {
        return self.userless;
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            allow_list: vec![],
            userless: false,
        }
    }
}