use crate::state::{read_config, Config};
use basset::reward::AccruedRewardsResponse;

use cosmwasm_std::{
    attr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};

pub fn execute_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    let custody_addr = config
        .custody_contract
        .clone()
        .ok_or_else(|| StdError::generic_err("the custody contract must have been registered"))?
        .to_string();

    // Only the custody contract can call this function
    if info.sender != deps.api.addr_validate(&custody_addr)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(info.sender);

    // This correspond exactly to the reward denom balance
    // We send a message if and only if the contract indeed has a denom balance
    let rewards: Coin = deps
        .querier
        .query_balance(env.contract.address, config.reward_denom)?;

    if rewards.amount.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet"));
    }

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "claim_reward"),
            attr("rewards", rewards.to_string()),
        ])
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![rewards],
        })))
}

pub fn query_accrued_rewards(
    deps: Deps,
    env: Env,
    _address: String,
) -> StdResult<AccruedRewardsResponse> {
    let config = read_config(deps.storage)?;
    let rewards = match deps
        .querier
        .query_balance(env.contract.address, config.reward_denom)
    {
        Err(_) => Uint128::zero(),
        Ok(c) => c.amount,
    };

    Ok(AccruedRewardsResponse { rewards })
}
