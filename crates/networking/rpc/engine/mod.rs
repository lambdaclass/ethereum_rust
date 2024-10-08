pub mod exchange_transition_config;
pub mod fork_choice;
pub mod payload;

use crate::{RpcErr, RpcHandler, Store};
use serde_json::{json, Value};

pub type ExchangeCapabilitiesRequest = Vec<String>;

impl RpcHandler for ExchangeCapabilitiesRequest {
    fn parse(params: &Option<Vec<Value>>) -> Result<Self, RpcErr> {
        params
            .as_ref()
            .ok_or(RpcErr::BadParams("No params provided".to_owned()))?
            .first()
            .ok_or(RpcErr::BadParams("Expected 1 param".to_owned()))
            .and_then(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|error| RpcErr::BadParams(error.to_string()))
            })
    }

    fn handle(&self, _storage: Store) -> Result<Value, RpcErr> {
        Ok(json!(*self))
    }
}
