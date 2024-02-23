use std::collections::HashSet;

use cosmwasm_std::{Addr, StdError, Empty};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::global::execute_swap;
use crate::state::{
    read_config, store_config, store_state, Config, State, SwapConfig, SWAP_CONFIG,
};
use crate::user::{execute_claim_rewards, query_accrued_rewards};
use cosmwasm_std::{
    to_binary, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use basset::reward::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn has_unique_elements(list: &[String]) -> bool {
    let mut uniq = HashSet::new();
    list.iter().all(move |x| uniq.insert(x))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    if !has_unique_elements(&msg.known_tokens) {
        return Err(StdError::generic_err(
            "Known tokens shouldn't contain duplicate assets",
        ));
    }

    let conf = Config {
        owner: info.sender,
        hub_contract: deps.api.addr_validate(&msg.hub_contract)?,
        custody_contract: None,
        reward_denom: msg.reward_denom,

        known_cw20_tokens: msg
            .known_tokens
            .iter()
            .map(|addr| deps.api.addr_validate(addr))
            .collect::<StdResult<Vec<Addr>>>()?,
    };

    store_config(deps.storage, &conf)?;
    store_state(
        deps.storage,
        &State {
            global_index: Decimal256::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    SWAP_CONFIG.save(
        deps.storage,
        &SwapConfig {
            astroport_addr: deps.api.addr_validate(&msg.astroport_addr)?,
            phoenix_addr: deps.api.addr_validate(&msg.phoenix_addr)?,
            terraswap_addr: deps.api.addr_validate(&msg.terraswap_addr)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => execute_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::SwapToRewardDenom {} => execute_swap(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            custody_contract,
            known_tokens,
            astroport_addr,
            phoenix_addr,
            terraswap_addr,
        } => set_custody_contract(
            deps,
            info,
            owner,
            custody_contract,
            known_tokens,
            astroport_addr,
            phoenix_addr,
            terraswap_addr,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn set_custody_contract(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    custody_contract: Option<String>,
    known_tokens: Option<Vec<String>>,
    astroport_addr: Option<String>,
    phoenix_addr: Option<String>,
    terraswap_addr: Option<String>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;
    let mut swap_config = SWAP_CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(custody_contract) = custody_contract {
        config.custody_contract = Some(deps.api.addr_validate(&custody_contract)?);
    }

    if let Some(astroport_addr) = astroport_addr {
        swap_config.astroport_addr = deps.api.addr_validate(&astroport_addr)?;
    }

    if let Some(phoenix_addr) = phoenix_addr {
        swap_config.phoenix_addr = deps.api.addr_validate(&phoenix_addr)?;
    }

    if let Some(terraswap_addr) = terraswap_addr {
        swap_config.terraswap_addr = deps.api.addr_validate(&terraswap_addr)?;
    }

    if let Some(known_tokens) = known_tokens {
        if !has_unique_elements(&known_tokens) {
            return Err(StdError::generic_err(
                "Known tokens shouldn't contain duplicate assets",
            ));
        }
        config.known_cw20_tokens = known_tokens
            .iter()
            .map(|token| deps.api.addr_validate(token))
            .collect::<StdResult<Vec<Addr>>>()?
    }

    store_config(deps.storage, &config)?;
    SWAP_CONFIG.save(deps.storage, &swap_config)?;

    Ok(Response::new().add_attribute("action", "set_custody_contract"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::AccruedRewards { address } => {
            to_binary(&query_accrued_rewards(deps, env, address)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = read_config(deps.storage)?;
    Ok(config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}
