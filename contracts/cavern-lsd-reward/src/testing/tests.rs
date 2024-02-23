//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(&[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(deps.as_ref(), mock_env(),...) you must replace it with query(&mut deps, ...)

use basset::dex_router::AssetInfo;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, StdError, Empty};
use cosmwasm_std::{from_binary, BankMsg, Coin, CosmosMsg, SubMsg, Uint128};

use crate::contract::{execute, instantiate, migrate, query};

use crate::swap::{create_swap_msgs, Asset};
use crate::testing::mock_querier::{mock_dependencies, MOCK_HUB_CONTRACT_ADDR};
use basset::reward::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

const DEFAULT_REWARD_DENOM: &str = "uusd";

fn default_init() -> InstantiateMsg {
    InstantiateMsg {
        hub_contract: String::from(MOCK_HUB_CONTRACT_ADDR),
        reward_denom: DEFAULT_REWARD_DENOM.to_string(),
        astroport_addr: "astroport_addr".to_string(),
        phoenix_addr: "phoenix_addr".to_string(),
        terraswap_addr: "terraswap_addr".to_string(),

        known_tokens: vec![],
    }
}

#[test]
fn proper_init() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = default_init();

    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            hub_contract: String::from(MOCK_HUB_CONTRACT_ADDR),
            reward_denom: DEFAULT_REWARD_DENOM.to_string(),
        }
    );
}

#[test]
pub fn swap_to_reward_denom() {
    let mut deps = mock_dependencies(&[
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(100u128),
        },
        Coin {
            denom: "ukrw".to_string(),
            amount: Uint128::new(1000u128),
        },
        Coin {
            denom: "usdr".to_string(),
            amount: Uint128::new(50u128),
        },
        Coin {
            denom: "mnt".to_string(),
            amount: Uint128::new(50u128),
        },
        Coin {
            denom: "uinr".to_string(),
            amount: Uint128::new(50u128),
        },
    ]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    let info = mock_info(String::from(MOCK_HUB_CONTRACT_ADDR).as_str(), &[]);
    let msg = ExecuteMsg::SwapToRewardDenom {};

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(
                create_swap_msgs(
                    deps.as_ref(),
                    mock_env(),
                    Asset {
                        amount: Uint128::new(1000u128),
                        asset_info: AssetInfo::NativeToken {
                            denom: "ukrw".to_string(),
                        }
                    },
                    DEFAULT_REWARD_DENOM.to_string()
                )
                .unwrap()[0]
                    .clone()
            ),
            SubMsg::new(
                create_swap_msgs(
                    deps.as_ref(),
                    mock_env(),
                    Asset {
                        amount: Uint128::new(50u128),
                        asset_info: AssetInfo::NativeToken {
                            denom: "usdr".to_string(),
                        }
                    },
                    DEFAULT_REWARD_DENOM.to_string()
                )
                .unwrap()[0]
                    .clone()
            ),
            SubMsg::new(
                create_swap_msgs(
                    deps.as_ref(),
                    mock_env(),
                    Asset {
                        amount: Uint128::new(50u128),
                        asset_info: AssetInfo::NativeToken {
                            denom: "uinr".to_string(),
                        }
                    },
                    DEFAULT_REWARD_DENOM.to_string()
                )
                .unwrap()[0]
                    .clone()
            ),
        ]
    );
}

#[test]
fn claim_rewards() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    deps.querier
        .base
        .update_balance(MOCK_CONTRACT_ADDR, coins(100u128, "uusd"));

    // claimed_rewards = 100, total_balance = 100
    // global_index == 1
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        ExecuteMsg::UpdateConfig {
            owner: None,
            custody_contract: Some("custody".to_string()),
            known_tokens: None,
            astroport_addr: None,
            phoenix_addr: None,
            terraswap_addr: None,
        },
    )
    .unwrap();

    let msg = ExecuteMsg::ClaimRewards { recipient: None };
    let info = mock_info("custody", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: String::from("custody"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100u128), // No tax fee
            },]
        }))]
    );

    // Set recipient
    // claimed_rewards = 100, total_balance = 100
    // global_index == 1

    let msg = ExecuteMsg::ClaimRewards {
        recipient: Some(String::from("addr0001")),
    };
    let info = mock_info("addr0000", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, StdError::generic_err("unauthorized"));
}

#[test]
fn claim_rewards_with_decimals() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(99999u128),
    }]);

    let init_msg = default_init();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    deps.querier
        .base
        .update_balance(MOCK_CONTRACT_ADDR, coins(99998u128, "uusd"));

    // claimed_rewards = 1000000, total_balance = 11
    // global_index ==

    // Setting the custody contract
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        ExecuteMsg::UpdateConfig {
            owner: None,
            custody_contract: Some("custody".to_string()),
            known_tokens: None,
            astroport_addr: None,
            phoenix_addr: None,
            terraswap_addr: None,
        },
    )
    .unwrap();

    let msg = ExecuteMsg::ClaimRewards {
        recipient: Some("addr0000".to_string()),
    };
    let info = mock_info("custody", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: String::from("addr0000"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(99998u128), // No tax
            },]
        }))]
    );
}

#[test]
fn test_migrate() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(100u128),
    }]);

    migrate(deps.as_mut(), mock_env(), Empty{}).unwrap();
}
