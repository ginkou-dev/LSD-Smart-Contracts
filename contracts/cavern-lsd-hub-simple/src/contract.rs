use cosmwasm_std::entry_point;
use cosmwasm_std::Empty;
use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg,
};

use crate::config::execute_update_config;

use crate::state::{CONFIG, STATE};

use basset::hub::{
    Config, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, State, StateResponse,
};

use basset::wrapper::ExecuteMsg as LSDWrapperExecuteMsg;
use basset::reward::ExecuteMsg::SwapToRewardDenom;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // store config
    let data = Config {
        creator: info.sender,
        reward_contract: None,
        token_contract: None,
    };
    CONFIG.save(deps.storage, &data)?;

    // store state
    let state = State {
        exchange_rate: Decimal::one(),
        last_index_modification: env.block.time.seconds(),
        ..Default::default()
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateGlobalIndex {} => execute_update_global(deps, env),
        // No need to check whether slashing has happened in our case.
        /*
        ExecuteMsg::CheckSlashing {} => execute_slashing(deps, env),
        */
        ExecuteMsg::UpdateConfig {
            owner,
            reward_contract,
            token_contract,
        } => execute_update_config(deps, env, info, owner, reward_contract, token_contract),
    }
}

/// Update general parameters
/// Permissionless
pub fn execute_update_global(
    deps: DepsMut,
    env: Env,
    //airdrop_hooks: Option<Vec<Binary>>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let reward_addr = config
        .reward_contract
        .clone()
        .ok_or_else(|| StdError::generic_err("the reward contract must have been registered"))?
        .to_string();

    let lsd_wrapper_contract = config
        .token_contract
        .ok_or_else(|| StdError::generic_err("the token contract must have been registered"))?
        .to_string();

    // Send decompound message so that LSD rewards get taken out of the token if they exist
    let decompound_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lsd_wrapper_contract,
        msg: to_binary(&LSDWrapperExecuteMsg::Decompound {
            recipient: Some(reward_addr.clone()),
        })?,
        funds: vec![],
    });

    //update state last modified
    STATE.update(deps.storage, |mut last_state| -> StdResult<State> {
        last_state.last_index_modification = env.block.time.seconds();
        Ok(last_state)
    })?;

    Ok(Response::new()
        .add_message(decompound_msg)
        .add_attributes(vec![attr("action", "update_global_index")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::Parameters {  } => panic!("No such query")
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut reward: Option<String> = None;
    let mut token: Option<String> = None;
    if config.reward_contract.is_some() {
        reward = Some(config.reward_contract.unwrap().to_string());
    }
    if config.token_contract.is_some() {
        token = Some(config.token_contract.unwrap().to_string());
    }

    Ok(ConfigResponse {
        owner: config.creator.to_string(),
        reward_contract: reward,
        token_contract: token,
        //airdrop_registry_contract: airdrop,
    })
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    let res = StateResponse {
        exchange_rate: state.exchange_rate,
        total_bond_amount: state.total_bond_amount,
        last_index_modification: state.last_index_modification,
        prev_hub_balance: state.prev_hub_balance,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}
