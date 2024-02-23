use basset::external::RewardInfoResponse;
use cosmwasm_std::Reply;
use cosmwasm_std::Binary;
use astroport::asset::PairInfo;
use astroport::pair::QueryMsg as PairQueryMsg;
use basset::custom_reward::InstantiateMsg;
use basset::external::SpectrumExecuteMsg;
use basset::external::SpectrumQueryMsg;

use anchor_basset_custom_reward::state::read_retrieve_config;
use basset::custom_reward::ExecuteWithSwapReply;

use anchor_basset_custom_reward::contract;
use astroport::pair::Cw20HookMsg;
use basset::reward::QueryMsg;
use basset::reward::ExecuteMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::DepsMut;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Env, MessageInfo, QueryRequest, Response, StdResult,
    WasmQuery, entry_point
};
use cw20::Cw20ExecuteMsg;

#[cfg(test)]
mod tests;

#[cw_serde]
pub struct RetrieveConfigRaw {
    spectrum_token: String,
    pair: String,
}

#[cw_serde]
pub struct RetrieveConfig {
    spectrum_token: Addr,
    pair: Addr,
}

pub struct SpectrumRetrieve {}
impl ExecuteWithSwapReply for SpectrumRetrieve {
    fn get_retrieve_messages(deps: Deps, env: Env) -> StdResult<Vec<CosmosMsg>> {
        let config = read_retrieve_config::<RetrieveConfig>(deps.storage)?;

        // 1. We unstake the LP tokens from Spectrum Protocol
        // https://github.com/spectrumprotocol/spectrum-core/blob/main/contracts/astroport_farm/src/bond.rs#L256
        let lp_amount: RewardInfoResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.spectrum_token.to_string(),
                msg: to_binary(&SpectrumQueryMsg::RewardInfo {
                    staker_addr: env.contract.address.to_string(),
                })?,
            }))?;

        if lp_amount.reward_info.bond_share.is_zero() {
            return Ok(vec![]);  
        }

        let pair_info: PairInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.pair.to_string(),
            msg: to_binary(&PairQueryMsg::Pair {  } )?,
        }))?;

        let mut messages = vec![CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.spectrum_token.to_string(),
            msg: to_binary(&SpectrumExecuteMsg::Unbond{
                amount: lp_amount.reward_info.bond_amount
            })?,
            funds: vec![],
        })];

        // 2. We send the liquidity token to the pair and use a withdraw liquidity message
        messages.push(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: pair_info.liquidity_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                amount: lp_amount.reward_info.bond_amount,
                contract: config.pair.to_string(),
                msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets: vec![] })?,
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
        let spectrum_token = deps.api.addr_validate(&config.spectrum_token)?;
        Ok(RetrieveConfig {
            spectrum_token,
            pair,
        })
    }

    type RetrieveConfigRaw = RetrieveConfigRaw;
    type RetrieveConfig = RetrieveConfig;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg<SpectrumRetrieve>) -> StdResult<Response> {
    contract::instantiate::<SpectrumRetrieve>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    contract::execute::<SpectrumRetrieve>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
   contract::reply(deps, env, msg)
}
