use cosmwasm_std::MessageInfo;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::Env;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;

pub trait LSDHub<I: for<'a> Deserialize<'a> + Serialize> {
    fn instantiate_config(deps: Deps, t: I) -> StdResult<Self>
    where
        Self: std::marker::Sized;
    fn query_exchange_rate(&self, deps: Deps, env: Env) -> StdResult<Decimal>;
    fn get_balance(&self, deps: Deps, env: Env, address: Addr) -> StdResult<Uint128>;
    fn deposit_funds(
        &self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        from: Addr,
    ) -> StdResult<Vec<CosmosMsg>>;
    fn send_funds(
        &self,
        deps: Deps,
        env: Env,
        amount: Uint128,
        to: Addr,
    ) -> StdResult<Vec<CosmosMsg>>;
}
