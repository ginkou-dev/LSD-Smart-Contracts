use cosmwasm_std::Uint128;
use basset::eris_lp::StateResponse;

use cosmwasm_std::Reply;
use cosmwasm_std::Binary;
use astroport::asset::PairInfo;
use astroport::pair::QueryMsg as PairQueryMsg;
use basset::custom_reward::InstantiateMsg;

use anchor_basset_custom_reward::state::read_retrieve_config;
use basset::custom_reward::ExecuteWithSwapReply;

use anchor_basset_custom_reward::contract;
use basset::reward::QueryMsg;
use basset::reward::{ExecuteMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::DepsMut;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Env, MessageInfo, QueryRequest, Response, StdResult,
    WasmQuery, entry_point
};
use cw20::BalanceResponse;
use cw20::Cw20ExecuteMsg;
use cw20::Cw20QueryMsg;

#[cfg(test)]
mod tests;

#[cw_serde]
pub struct RetrieveConfigRaw {
    amp_lp_token: String,
    hub: String,
    pair: String,
}

#[cw_serde]
pub struct RetrieveConfig {
    amp_lp_token: Addr,
    hub: Addr,
    pair: Addr,
}

pub struct AmpLPRetrieve {}
impl ExecuteWithSwapReply for AmpLPRetrieve {
    fn get_retrieve_messages(deps: Deps, env: Env) -> StdResult<Vec<CosmosMsg>> {
        let config = read_retrieve_config::<RetrieveConfig>(deps.storage)?;

        // 1. We unstake the LP tokens from Eris Protocol
        // https://github.com/erisprotocol/contracts-terra/blob/main/contracts/amp-compounder/astroport_farm/src/contract.rs#L166
        let amp_lp_balance: BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.amp_lp_token.to_string(),
                msg: to_binary(&Cw20QueryMsg::Balance { address: env.contract.address.to_string() })?,
            }))?;
        // TODO
        if amp_lp_balance.balance.is_zero() {
            return Ok(vec![]);
        }

        let mut messages = vec![CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.amp_lp_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send{
                contract: config.hub.to_string(),
                msg: to_binary(&basset::eris_lp::Cw20HookMsg::Unbond{receiver: None})?,
                amount: amp_lp_balance.balance,
            })?,
            funds: vec![],
        })];

        // 2. We send the liquidity token to the pair and use a withdraw liquidity message (astroport)

        let pair_info: PairInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.pair.to_string(),
            msg: to_binary(&PairQueryMsg::Pair {  } )?,
        }))?;
        // First we need to compute how much tokens we will have in store after the operation
        // a. We get the current lp balance (for getting past amounts)
        let astro_lp_balance : BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: pair_info.liquidity_token.to_string(),
                msg: to_binary(&Cw20QueryMsg::Balance { address: env.contract.address.to_string() })?,
            }))?;

        // b. We get the computation of how much tokens should be exchanged
        let state : StateResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.hub.to_string(),
                msg: to_binary(&basset::eris_lp::QueryMsg::State {
                    addr: None
                })?,
            }))?;

        let bond_amount =  if state.total_amp_lp.is_zero() {
            Uint128::zero()
        } else {
            state.total_lp.multiply_ratio(amp_lp_balance.balance, state.total_amp_lp)
        };

        if(bond_amount + astro_lp_balance.balance).is_zero(){
            return Ok(vec![])
        }

        messages.push(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: pair_info.liquidity_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                amount: bond_amount + astro_lp_balance.balance,
                contract: config.pair.to_string(),
                msg: to_binary(&astroport::pair::Cw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
            })?,
            funds: vec![],
        }));

        // We send the liquidity token to the pair and use a withdraw liquidity message
        Ok(messages)
    }

    fn validate_retrieve_config(
        deps: Deps,
        config: RetrieveConfigRaw,
    ) -> StdResult<RetrieveConfig> {
        let pair = deps.api.addr_validate(&config.pair)?;
        let amp_lp_token = deps.api.addr_validate(&config.amp_lp_token)?;
        let hub = deps.api.addr_validate(&config.hub)?;
        Ok(RetrieveConfig {
            amp_lp_token,
            hub,
            pair,
        })
    }

    type RetrieveConfigRaw = RetrieveConfigRaw;
    type RetrieveConfig = RetrieveConfig;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg<AmpLPRetrieve>) -> StdResult<Response> {
    contract::instantiate::<AmpLPRetrieve>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    contract::execute::<AmpLPRetrieve>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
   contract::reply(deps, env, msg)
}
