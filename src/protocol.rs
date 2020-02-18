use std::fmt;

use serde::{Deserialize};
use schemars::JsonSchema;

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct ServiceCoordinates {
    pub name: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub port: i32,
}

fn default_namespace() -> String {
    "default".to_string()
}

#[derive(Clone, Debug)]
pub struct SingularEndpoint {
    pub ip: String,
    pub port: i32,
}

impl fmt::Display for SingularEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}
