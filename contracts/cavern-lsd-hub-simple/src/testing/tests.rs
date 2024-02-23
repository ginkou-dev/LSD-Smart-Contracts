//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(deps.as_ref(), ...) you must replace it with query(&mut deps, ...)

use cosmwasm_schema::cw_serde;
use cosmwasm_std::coins;
use cosmwasm_std::{
    coin, from_binary, to_binary, Api, CosmosMsg, Decimal, OwnedDeps, Querier, Response, StdError,
    Storage, Uint128, WasmMsg,
};

use cosmwasm_std::testing::{mock_env, mock_info};

use crate::contract::{execute, instantiate, query};
use basset::hub::{QueryMsg, Parameters};
use basset::hub::{ConfigResponse, ExecuteMsg, InstantiateMsg, StateResponse};

use basset::hub::ExecuteMsg::UpdateConfig;

use super::mock_querier::mock_dependencies as dependencies;


use basset::reward::ExecuteMsg::SwapToRewardDenom;
use std::borrow::BorrowMut;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut OwnedDeps<S, A, Q>,
    owner: String,
    reward_contract: String,
    token_contract: String,
) {
    let msg = InstantiateMsg {};

    let owner_info = mock_info(owner.as_str(), &[coin(1000000, "uluna")]);
    instantiate(deps.as_mut(), mock_env(), owner_info.clone(), msg).unwrap();

    let register_msg = ExecuteMsg::UpdateConfig {
        owner: None,
        reward_contract: Some(reward_contract),
        token_contract: Some(token_contract),
    };

    let res = execute(deps.as_mut(), mock_env(), owner_info, register_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

/// Covers if all the fields of InitMsg are stored in
/// parameters' storage, the config storage stores the creator,
/// the current batch storage and state are initialized.
#[test]
fn proper_initialization() {
    let mut deps = dependencies(&[]);

    // successful call
    let msg = InstantiateMsg {};

    let _owner = "owner1";
    let owner_info = mock_info("owner1", &[]);

    // we can just call .unwrap() to assert this was a success
    let res: Response = instantiate(deps.as_mut(), mock_env(), owner_info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // state storage must be initialized
    let state = QueryMsg::State {};
    let query_state: StateResponse =
        from_binary(&query(deps.as_ref(), mock_env(), state).unwrap()).unwrap();
    let expected_result = StateResponse {
        exchange_rate: Decimal::one(),
        total_bond_amount: Uint128::zero(),
        last_index_modification: mock_env().block.time.seconds(),
        prev_hub_balance: Default::default(),
    };
    assert_eq!(query_state, expected_result);

    // config storage must be initialized
    let conf = QueryMsg::Config {};
    let query_conf: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), conf).unwrap()).unwrap();
    let expected_conf = ConfigResponse {
        owner: "owner1".to_string(),
        reward_contract: None,
        token_contract: None,
        //airdrop_registry_contract: None,
    };

    assert_eq!(expected_conf, query_conf);
}

/// Covers if Withdraw message, swap message, and update global index are sent.
#[test]
pub fn proper_update_global_index() {
    let mut deps = dependencies(&[]);

    let addr1 = "addr1000".to_string();
    let bond_amount = Uint128::new(10);

    let owner = "owner1".to_string();
    let token_contract = "token".to_string();
    let reward_contract = "reward".to_string();

    init(
        deps.borrow_mut(),
        owner,
        reward_contract.clone(),
        token_contract.clone(),
    );

    // fails if there is no delegation
    let reward_msg = ExecuteMsg::UpdateGlobalIndex {};

    let info = mock_info(&addr1, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, reward_msg).unwrap();
    assert_eq!(res.messages.len(), 2);

    deps.querier
        .base
        .update_balance(MOCK_CONTRACT_ADDR, coins(10, "uusd"));

    //set bob's balance to 10 in token contract
    deps.querier
        .with_token_balances(&[(&"token".to_string(), &[(&addr1, &bond_amount)])]);

    let reward_msg = ExecuteMsg::UpdateGlobalIndex {
        //airdrop_hooks: None,
    };

    let info = mock_info(&addr1, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, reward_msg).unwrap();
    assert_eq!(2, res.messages.len());

    let last_index_query = QueryMsg::State {};
    let last_modification: StateResponse =
        from_binary(&query(deps.as_ref(), mock_env(), last_index_query).unwrap()).unwrap();
    assert_eq!(
        &last_modification.last_index_modification,
        &mock_env().block.time.seconds()
    );

    let withdraw = &res.messages[0].msg;
    match withdraw {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: lsd_wrapper_contract,
            msg: _,
            funds: _,
        }) => {
            assert_eq!(lsd_wrapper_contract.clone(), token_contract);
        }
        _ => panic!("Unexpected message: {:?}", withdraw),
    }

    let swap = &res.messages[1].msg;
    match swap {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) => {
            assert_eq!(contract_addr, &reward_contract);
            assert_eq!(msg, &to_binary(&SwapToRewardDenom {}).unwrap())
        }
        _ => panic!("Unexpected message: {:?}", swap),
    }
}

/// Covers if the storage affected by update_config are updated properly
#[test]
pub fn proper_update_config() {
    let mut deps = dependencies(&[]);

    let owner = "owner1".to_string();
    let new_owner = "new_owner".to_string();
    let invalid_owner = "invalid_owner".to_string();
    let token_contract = "token".to_string();
    let reward_contract = "reward".to_string();

    init(
        &mut deps,
        owner.clone(),
        reward_contract.clone(),
        token_contract.clone(),
    );

    let config = QueryMsg::Config {};
    let config_query: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), config).unwrap()).unwrap();
    assert_eq!(&config_query.token_contract.unwrap(), &token_contract);

    //make sure the other configs are still the same.
    assert_eq!(&config_query.reward_contract.unwrap(), &reward_contract);
    assert_eq!(&config_query.owner, &owner);

    // only the owner can call this message
    let update_config = UpdateConfig {
        owner: Some(new_owner.clone()),
        reward_contract: None,
        token_contract: None,
    };
    let info = mock_info(&invalid_owner, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, update_config);
    assert_eq!(res.unwrap_err(), StdError::generic_err("unauthorized"));

    // only the owner can call this message
    let update_config = UpdateConfig {
        owner: Some(new_owner.clone()),
        reward_contract: None,
        token_contract: None,
    };
    let info = mock_info(&owner, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, update_config).unwrap();

    let update_config = UpdateConfig {
        owner: None,
        reward_contract: Some("new reward".to_string()),
        token_contract: None,
    };
    let new_owner_info = mock_info(new_owner.as_ref(), &[]);
    let res = execute(deps.as_mut(), mock_env(), new_owner_info, update_config).unwrap();
    assert_eq!(res.messages.len(), 0);

    let config = QueryMsg::Config {};
    let config_query: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), config).unwrap()).unwrap();
    assert_eq!(
        config_query.reward_contract.unwrap(),
        "new reward".to_string()
    );

    let update_config = UpdateConfig {
        owner: None,
        reward_contract: None,
        token_contract: Some("new token".to_string()),
    };
    let new_owner_info = mock_info(new_owner.as_ref(), &[]);
    let res = execute(deps.as_mut(), mock_env(), new_owner_info, update_config).unwrap();
    assert_eq!(res.messages.len(), 0);

    let config = QueryMsg::Config {};
    let config_query: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), config).unwrap()).unwrap();
    assert_eq!(
        config_query.token_contract.unwrap(),
        "new token".to_string()
    );

    //make sure the other configs are still the same.
    assert_eq!(
        config_query.reward_contract.unwrap(),
        "new reward".to_string()
    );
    assert_eq!(config_query.owner, new_owner);

    let update_config = UpdateConfig {
        owner: None,
        reward_contract: None,
        token_contract: None,
    };
    let new_owner_info = mock_info(new_owner.as_ref(), &[]);
    let res = execute(deps.as_mut(), mock_env(), new_owner_info, update_config).unwrap();
    assert_eq!(res.messages.len(), 0);
}

// sample MIR claim msg
#[cw_serde]
pub enum MIRMsg {
    MIRClaim {},
}
