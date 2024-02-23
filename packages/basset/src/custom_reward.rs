use cosmwasm_schema::cw_serde;
use cosmwasm_std::Env;
use cosmwasm_std::{CosmosMsg, Deps, Empty, StdResult};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

pub trait ExecuteWithSwapReply {
    type RetrieveConfigRaw: JsonSchema;
    type RetrieveConfig: Serialize + DeserializeOwned;
    fn get_retrieve_messages(deps: Deps, env: Env) -> StdResult<Vec<CosmosMsg>>;
    fn validate_retrieve_config(
        deps: Deps,
        config: Self::RetrieveConfigRaw,
    ) -> StdResult<Self::RetrieveConfig>;
}

// For tests, we implement for the empty object
impl ExecuteWithSwapReply for Empty {
    type RetrieveConfig = Option<Empty>;
    type RetrieveConfigRaw = Option<Empty>;
    fn get_retrieve_messages(
        _deps: cosmwasm_std::Deps,
        _env: Env,
    ) -> cosmwasm_std::StdResult<Vec<CosmosMsg>> {
        Ok(vec![])
    }
    fn validate_retrieve_config(
        _deps: Deps,
        _config: Self::RetrieveConfigRaw,
    ) -> cosmwasm_std::StdResult<Self::RetrieveConfig> {
        Ok(None)
    }
}

#[cw_serde]
pub struct InstantiateMsg<T: ExecuteWithSwapReply> {
    pub hub_contract: String,
    pub reward_denom: String,
    pub astroport_addr: String,
    pub phoenix_addr: String,
    pub terraswap_addr: String,
    // Known tokens to swap from to the stable_token
    pub known_tokens: Vec<String>,

    pub retrieve_config: T::RetrieveConfigRaw,
}
