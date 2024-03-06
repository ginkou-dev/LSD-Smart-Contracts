use crate::querier::get_lsd_wrapper_exchange_rate;
use crate::state::read_lsd_config;
use crate::trait_def::LSDHub;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;

use cosmwasm_std::{Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};
use cw20_base::contract::query_balance;

use serde::Deserialize;
use serde::Serialize;

use cw20_base::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_base::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer,
};
use cw20_base::ContractError;

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer(deps, env, info, recipient, amount)
}

fn _before_burn<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let lsd_config: T = read_lsd_config(deps.storage)?;
    // When burning some tokens from here, we transfer an equivalent amount of 1 Luna per each burned token to the burner
    let lsd_exchange_rate = get_lsd_wrapper_exchange_rate::<I, T>(deps, env.clone(), vec![])?;
    let lsd_amount = Decimal::from_ratio(amount, 1u128) / lsd_exchange_rate;

    let msgs = lsd_config.send_funds(deps, env, lsd_amount * Uint128::one(), info.sender)?;
    Ok(msgs)
}

pub fn execute_burn<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let transfer_messages = _before_burn::<I, T>(deps.as_ref(), env.clone(), info.clone(), amount)?;

    let res = cw20_burn(deps, env, info, amount)?;

    Ok(res.add_messages(transfer_messages))
}

pub fn execute_burn_all<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let amount = query_balance(deps.as_ref(), info.sender.to_string())?;

    if amount.balance.is_zero() {
        return Ok(Response::new());
    }
    execute_burn::<I, T>(deps, env, info, amount.balance)
}

pub fn execute_mint<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // In order to mint, we need to transfer the underlying lsd asset to the contract
    // Any sender can call this function as long as they have the sufficient lsd balance
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let exchange_rate =
        get_lsd_wrapper_exchange_rate::<I, T>(deps.as_ref(), env.clone(), info.funds.clone())?;
    // We add 1 to the send_lsd_amount here to make sure we are not undercollateralizing our token at the start
    let send_lsd_amount = Decimal::from_ratio(amount, 1u128) / exchange_rate + Decimal::one();

    let messages = lsd_config.deposit_funds(
        deps.as_ref(),
        env.clone(),
        info.clone(),
        send_lsd_amount * Uint128::one(),
        info.sender,
    )?;
    info.sender = env.contract.address.clone();

    let res = cw20_mint(deps, env, info, recipient, amount)?;

    Ok(res.add_messages(messages))
}

pub fn execute_mint_with<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    recipient: String,
    lsd_amount: Uint128,
) -> Result<Response, ContractError> {
    // In order to mint, we need to transfer the underlying lsd asset to the contract
    // Any sender can call this function as long as they have the sufficient lsd balance
    let lsd_config: T = read_lsd_config(deps.storage)?;
    let exchange_rate =
        get_lsd_wrapper_exchange_rate::<I, T>(deps.as_ref(), env.clone(), info.funds.clone())?;
    let mint_amount = lsd_amount * exchange_rate;

    let messages = lsd_config.deposit_funds(
        deps.as_ref(),
        env.clone(),
        info.clone(),
        lsd_amount,
        info.sender,
    )?;

    info.sender = env.contract.address.clone();

    let res = cw20_mint(deps, env, info, recipient, mint_amount * Uint128::one())?;

    Ok(res.add_messages(messages))
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send(deps, env, info, contract, amount, msg)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer_from(deps, env, info, owner, recipient, amount)
}

pub fn execute_burn_from<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let transfer_messages = _before_burn::<I, T>(deps.as_ref(), env.clone(), info.clone(), amount)?;

    let res = cw20_burn_from(deps, env, info, owner, amount)?;

    Ok(res.add_messages(transfer_messages))
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send_from(deps, env, info, owner, contract, amount, msg)
}
