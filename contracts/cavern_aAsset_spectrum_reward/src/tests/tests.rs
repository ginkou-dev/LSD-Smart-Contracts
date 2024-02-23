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

use crate::tests::mock_querier::MOCK_ASTROPORT_LP_TOKEN;
use astroport::pair::Cw20HookMsg;

use anchor_basset_custom_reward::contract::RETRIEVE_NORMAL_TOKENS_OPERATION;

use cw20::Cw20ExecuteMsg;
use crate::{instantiate, execute, query, SpectrumRetrieve, RetrieveConfigRaw};

use basset::external::{SpectrumExecuteMsg};
use cosmwasm_std::testing::{mock_env, mock_info};

use cosmwasm_std::{from_binary, Coin, SubMsg, Uint128, CosmosMsg, to_binary};


use crate::tests::mock_querier::{mock_dependencies, MOCK_HUB_CONTRACT_ADDR};
use basset::custom_reward::InstantiateMsg;
use basset::reward::{ConfigResponse, ExecuteMsg, QueryMsg};

use super::mock_querier::{MOCK_ASTROPORT_PAIR, MOCK_SPECTRUM_TOKEN};

const DEFAULT_REWARD_DENOM: &str = "uusd";

fn default_init() -> InstantiateMsg<SpectrumRetrieve> {
    InstantiateMsg {
        hub_contract: String::from(MOCK_HUB_CONTRACT_ADDR),
        reward_denom: DEFAULT_REWARD_DENOM.to_string(),
        astroport_addr: "astroport_addr".to_string(),
        phoenix_addr: "phoenix_addr".to_string(),
        terraswap_addr: "terraswap_addr".to_string(),

        known_tokens: vec![],

        retrieve_config: RetrieveConfigRaw { 
            spectrum_token: MOCK_SPECTRUM_TOKEN.to_string(),
            pair: MOCK_ASTROPORT_PAIR.to_string(),
        }
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
                CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: MOCK_SPECTRUM_TOKEN.to_string(), msg: to_binary(&SpectrumExecuteMsg::Unbond{
                    amount: 5000000u128.into()
                }).unwrap(), funds: vec![] })
            ),
            SubMsg::reply_on_success( 
                CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: MOCK_ASTROPORT_LP_TOKEN.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        amount: 5000000u128.into(),
                        contract: MOCK_ASTROPORT_PAIR.to_string(),
                        msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets: vec![] }).unwrap(),
                }).unwrap(),
            funds: vec![],
        }), RETRIEVE_NORMAL_TOKENS_OPERATION),
            
        ]
    );
}