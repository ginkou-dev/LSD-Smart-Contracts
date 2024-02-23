use crate::querier::{get_current_exchange_rate, get_expected_exchange_rate, get_lsd_wrapper_decompound_rate, get_lsd_wrapper_exchange_rate};
use crate::state::read_lsd_config;
use crate::state::HUB_CONTRACT_KEY;
use crate::state::{DecompoundConfig, DecompoundState, DECOMPOUND_CONFIG, DECOMPOUND_STATE};
use basset::wrapper::{AccruedRewardsLimited, TokenInfoResponseWithLimit, MintAmountReponseWithLimit, GetExpectedExchangeRateResponse};
use cw20_base::enumerable::{query_all_accounts, query_owner_allowances, query_spender_allowances};
use serde::Serialize;
use crate::state::{ read_lsd_decompound_rate};
use crate::trait_def::LSDHub;
use basset::reward::MigrateMsg;

use cw20_base::contract::{query_balance, query_download_logo, query_marketing_info, query_minter, query_token_info};
use serde::Deserialize;

use crate::state::WrapperState;

use crate::state::store_hub_contract;
use crate::state::store_lsd_config;
use basset::wrapper::ExecuteMsg;
use cosmwasm_std::{entry_point, to_binary, attr};
use crate::querier::query_mint_amount;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use cw20_base::allowances::{execute_decrease_allowance, execute_increase_allowance, query_allowance};
use cw20_base::contract::query as cw20_query;
use cw20_base::contract::{
    execute_update_marketing, execute_update_minter, execute_upload_logo, instantiate as cw20_init,
};
use cw20_base::msg::InstantiateMsg;
use basset::wrapper::QueryMsg;
use crate::handler::*;
use crate::msg::TokenInitMsg;
use cw20::MinterResponse;
use cw20_base::ContractError;

pub const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

pub fn instantiate<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: TokenInitMsg<I>,
) -> StdResult<Response> {
    let lsd_config = T::instantiate_config(deps.as_ref(), msg.lsd_config)?;
    store_lsd_config(deps.storage, &lsd_config)?;

    store_hub_contract(deps.storage, &deps.api.addr_validate(&msg.hub_contract)?)?;

    DECOMPOUND_CONFIG.save(
        deps.storage,
        &DecompoundConfig {
            max_decompound_ratio: msg.max_decompound_ratio,
        },
    )?;

    DECOMPOUND_STATE.save(
        deps.storage,
        &DecompoundState {
            ratio_sum: Decimal::zero(),
            total_seconds: 0u64,
            last_decompound: env.block.time,
        },
    )?;

    cw20_init(
        deps,
        env.clone(),
        info,
        InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            mint: Some(MinterResponse {
                /// Only this contract can mint new tokens in exchange of the underlying lsd
                minter: env.contract.address.to_string(),
                cap: None,
            }),
            marketing: None,
        },
    )
    .map_err(|_| StdError::generic_err("CW20 Token init error"))?;

    Ok(Response::default())
}

pub fn execute<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + for<'a> Deserialize<'a> + Serialize,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn::<I, T>(deps, env, info, amount),
        ExecuteMsg::BurnAll {} => execute_burn_all::<I, T>(deps, env, info),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => {
            execute_mint::<I, T>(deps, env, info, recipient, amount)
        }
        ExecuteMsg::MintWith {
            recipient,
            lsd_amount,
        } => execute_mint_with::<I, T>(deps, env, info, recipient, lsd_amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => {
            execute_burn_from::<I, T>(deps, env, info, owner, amount)
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::UpdateMinter { new_minter } => {
            execute_update_minter(deps, env, info, new_minter)
        }
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
        ExecuteMsg::Decompound { recipient } => {
            execute_decompound::<I, T>(deps, env, info, recipient)
        }
    }
}

pub fn query<
    I: Serialize + for<'a> Deserialize<'a>,
    T: LSDHub<I> + Serialize + for<'b> Deserialize<'b>,
>(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    // If the token info are queried, we also add the current wLSD exchange rate to it
    match msg {
        QueryMsg::TokenInfo {} => {
            let token_info = query_token_info(deps)?;
            let mut state = WrapperState::default();
            let decompound_rate = read_lsd_decompound_rate(deps.storage)?;
            to_binary(&TokenInfoResponseWithLimit {
                name: token_info.name,
                symbol: token_info.symbol,
                decimals: token_info.decimals,
                total_supply: token_info.total_supply,
                exchange_rate: get_current_exchange_rate::<I, T>(deps, env.clone(), &mut state)
                    .map_err(|err| StdError::generic_err(err.to_string()))?,
                expected_exchange_rate: get_expected_exchange_rate::<I, T>(deps, env.clone(), &mut state)
                    .map_err(|err| StdError::generic_err(err.to_string()))?,
                    max_decompound_ratio: decompound_rate.max_decompound_ratio,
                    lsd_exchange_rate: get_lsd_wrapper_exchange_rate::<I, T>(deps, env.clone()).map_err(|err| StdError::generic_err(err.to_string()))?,
              //  max_decompound_ratio: get_lsd_wrapper_decompound_rate(deps, env)
               //     .map_err(|err| StdError::generic_err(err.to_string()))?,
            })
        },
        QueryMsg::GetMintAmount { amount } => {
            to_binary(&MintAmountReponseWithLimit {
                mint_amount: query_mint_amount::<I, T>(deps, env.clone(),amount).map_err(|err| StdError::generic_err(err.to_string()))?,
            })
        }
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_binary(&query_spender_allowances(
            deps,
            spender,
            start_after,
            limit,
        )?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
        QueryMsg::GetExpectedExchangeRate { } => {
            let mut state = WrapperState::default();
            to_binary(&GetExpectedExchangeRateResponse {
                expected_exchange_rate: get_expected_exchange_rate::<I, T>(deps, env.clone(), &mut state)
                .map_err(|err| StdError::generic_err(err.to_string()))?,            
            })
        },
    }
}

fn compute_accrued_rewards<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: Deps,
    env: Env,
) -> Result<AccruedRewardsLimited, ContractError> {
    // In this function, we have to make sure the token has a 1 exchange rate to Luna.
    let mut state = WrapperState::default();
    let current_exchange_rate = get_current_exchange_rate::<I, T>(deps, env.clone(), &mut state)?;

    // If the current exchange rate is lower than the previous one, we have just had a slashing event or something else
    // We can't decompound and we can't recompound
    if current_exchange_rate < Decimal::one() {
        // There is no accrued rewards to decompound.
        return Err(ContractError::Std(StdError::generic_err(
            "No rewards to decompound",
        )));
    }

    // Else, we have some available rewards to decompound
    let mut luna_rewards = state.backing_luna * Uint128::one() - state.wlsd_supply;

    let mut rewards_to_decompound = (Decimal::from_ratio(state.lsd_balance, 1u128)
        - (Decimal::from_ratio(state.wlsd_supply, 1u128) / state.lsd_exchange_rate))
        * Uint128::one();

    /******* Limiting the ratio of rewards extracted *********/

    let decompound_config = DECOMPOUND_CONFIG.load(deps.storage)?;
    if let Some(max_decompound_ratio) = decompound_config.max_decompound_ratio {
        // Then we want to limit the exchange rate to make sure we don't decompound too much from the LSD

        let decompound_state = DECOMPOUND_STATE.load(deps.storage)?;
        // SUM(ratio) / total_period < config.max_decompound_ratio / SECONDS_PER_YEAR
        // --> new_rate < config.max_decompound_ratio * total_period / SECONDS_PER_YEAR - Sum(old_rates)
        let total_period = decompound_state.total_seconds
            + env
                .block
                .time
                .seconds() - decompound_state.last_decompound.seconds();

        println!("total_period : {}",total_period);

        // If we decompounded too much in the past, this will simply error
        // As soon as the ratio goes back to normal, this will stop erroring
        let max_rate = (max_decompound_ratio * Decimal::from_ratio(total_period, SECONDS_PER_YEAR))
            .checked_sub(decompound_state.ratio_sum)
            .map_err(|_| {
                ContractError::Std(StdError::generic_err(
                    "Error substracting the total ratio to the current max_decompound ratio",
                ))
            })?;

        println!("max_rate : {}",max_rate);
        luna_rewards = luna_rewards.min(max_rate * state.backing_luna * Uint128::one());
        rewards_to_decompound = rewards_to_decompound.min(max_rate * state.lsd_balance);
    }

    /******* END  *********/

    // We substract 1 to the rewards to decompound so that we don't screw up the underlying value of the wrapper token
    // The underlying value should be if possible always above 1 luna per wrapper token (slashing events should happen as little often as possible)
    if rewards_to_decompound > Uint128::zero() {
        rewards_to_decompound -= Uint128::one();
        luna_rewards -= state.lsd_exchange_rate * Uint128::one();
    }

    let rate_decrease = Decimal::from_ratio(luna_rewards, 1u128) / state.backing_luna;

    Ok(AccruedRewardsLimited {
        rate_decrease,
        luna_rewards,
        lsd_rewards: rewards_to_decompound * Uint128::one(),
    })
}

/*
let luna_rewards = (wlsd_exchange_rate - 1) * wlsd_supply = current_luna_amount - wanted_luna_amount
let rewards_to_decompound = luna_rewards / lsd_exchange_rate = current_lsd_balance - wanted_lsd_balance


lr = (cr - 1) * wlsd
rtd = (cr - 1) * wlsd / exchange_rate

cr = b*exchange_rate / wlsd

lr = (b * exchange_rate - wlsd);
rtd = (b - wlsd/exchange_rate);

*/

pub fn execute_decompound<
    I: Serialize + for<'b> Deserialize<'b>,
    T: LSDHub<I> + Serialize + for<'a> Deserialize<'a>,
>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let hub_contract = HUB_CONTRACT_KEY.load(deps.storage)?;
    if info.sender != hub_contract {
        return Err(ContractError::Unauthorized {});
    }

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(info.sender);

    let lsd_config: T = read_lsd_config(deps.storage)?;
    let slashing_error = ContractError::Std(StdError::generic_err("No rewards to decompound"));
    let (out_messages, accrued_rewards) =
        match compute_accrued_rewards::<I, T>(deps.as_ref(), env.clone()) {
            Err(err) => {
                if err == slashing_error {
                    Ok((vec![], AccruedRewardsLimited::default()))
                } else {
                    Err(err)
                }
            }
            Ok(rewards) => {
                // We save the decreased rewards in the configuration for later usage
                let old_decompound = DECOMPOUND_STATE.load(deps.storage)?;

                if old_decompound.last_decompound >= env.block.time {
                    return Err(ContractError::Std(StdError::generic_err(
                        "Can't decompound too often",
                    )));
                }
                let new_decompound = DecompoundState {
                    ratio_sum: old_decompound.ratio_sum + rewards.rate_decrease,
                    total_seconds: old_decompound.total_seconds
                        + env.block.time.seconds() - old_decompound.last_decompound.seconds(),
                    last_decompound: env.block.time,
                };

                DECOMPOUND_STATE.save(deps.storage, &new_decompound)?;

                let decompound_messages = if !rewards.lsd_rewards.is_zero() {
                    lsd_config.send_funds(deps.as_ref(), env, rewards.lsd_rewards, recipient)?
                } else {
                    vec![]
                };
                Ok((decompound_messages, rewards))
            }
        }?;

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "execute_decompound"),
            attr(
                "total_luna_rewards",
                accrued_rewards.luna_rewards.to_string(),
            ),
        ])
        .add_messages(out_messages);

    Ok(res)
}

pub fn update_decompound_rate(
deps: DepsMut,
env: Env,
info: MessageInfo,
decompound_rate: Option<Decimal>,
) -> Result<Response, ContractError> {


        DECOMPOUND_CONFIG.save(
            deps.storage,
            &DecompoundConfig {
                max_decompound_ratio: decompound_rate,
            },
        )?;
        let res = Response::new()
            .add_attributes(vec![
                attr("action", "update_decompound_rate")
            ]);
        Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    // For the spectrum LP, we need to send some LP tokens back to the person that had an error

    // We update the max_decompound_ratio
    DECOMPOUND_CONFIG.save(
        deps.storage,
        &DecompoundConfig {
            max_decompound_ratio: msg.max_decompound_ratio,
        },
    )?;
/*
    DECOMPOUND_STATE.save(
        deps.storage,
        &DecompoundState { ratio_sum: Decimal::zero(), total_seconds: 0, last_decompound: env.block.time })?;

*/
    
    Ok(Response::default())
}


