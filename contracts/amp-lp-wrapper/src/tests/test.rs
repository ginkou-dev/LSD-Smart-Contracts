use crate::{execute, instantiate, query, ContractsRaw};
use basset::wrapper::ExecuteMsg;
use cavern_lsd_wrapper_token_with_limit::state::DECOMPOUND_CONFIG;
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::testing::{mock_info, MockApi};
use cosmwasm_std::to_binary;
use cosmwasm_std::MemoryStorage;
use cosmwasm_std::OwnedDeps;
use cosmwasm_std::{from_binary, CosmosMsg, SubMsg, WasmMsg};
use cosmwasm_std::{Decimal, Uint128, StdError};
use cw20::{BalanceResponse, Cw20ExecuteMsg};
use cw20_base::ContractError;
use std::str::FromStr;

use crate::tests::mock_deps::mock_dependencies;
use crate::tests::mock_deps::WasmMockQuerier;
use crate::SpectrumInitMsg;

const MOCK_SPECTRUM_TOKEN: &str = "spectrum-token";

fn get_transfer_from_msg<T: Into<String>, U: Into<String>, V: Into<String>, W: Into<Uint128>>(
    token: T,
    from: U,
    to: V,
    value: W,
) -> SubMsg {
    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.into(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: from.into(),
            recipient: to.into(),
            amount: value.into(),
        })
        .unwrap(),
        funds: vec![],
    }))
}

fn get_transfer_msg<T: Into<String>, V: Into<String>, W: Into<Uint128>>(
    token: T,
    to: V,
    value: W,
) -> SubMsg {
    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: to.into(),
            amount: value.into(),
        })
        .unwrap(),
        funds: vec![],
    }))
}

fn init_env(
    max_decompound_ratio: Option<&str>,
) -> OwnedDeps<MemoryStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies();
    let instantiate_msg = SpectrumInitMsg {
        decimals: 6,
        initial_balances: vec![],
        types: None,
        name: "spectrum-wrapper".to_string(),
        symbol: "w-S".to_string(),

        hub_contract: "hub".to_string(),

        lsd_config: ContractsRaw {
            token: MOCK_SPECTRUM_TOKEN.to_string(),
            hub: "astroport-hub".to_string(),
        },
        max_decompound_ratio: max_decompound_ratio.map(|v| Decimal::from_str(v).unwrap()),
    };
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        instantiate_msg,
    )
    .unwrap();

    // Init the token
    deps
}

#[test]
fn test_init_wrapper() {
    let deps = init_env(Some("0.1"));

    // We verify that the max_decompound_ratio is indeed registered
    assert_eq!(
        DECOMPOUND_CONFIG
            .load(&deps.storage)
            .unwrap()
            .max_decompound_ratio,
        Some(Decimal::from_str("0.10").unwrap())
    );
}

// Now we get to depositing
#[test]
fn test_deposit_funds() {
    let mut deps = init_env(Some("0.1"));

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("depositor", &[]),
        ExecuteMsg::MintWith {
            recipient: "depositor".to_string(),
            lsd_amount: 1_000_000u128.into(),
        },
    )
    .unwrap();
    // We assert the messages transfer the underyling asset
    assert_eq!(
        res.messages,
        vec![get_transfer_from_msg(
            MOCK_SPECTRUM_TOKEN,
            "depositor",
            MOCK_CONTRACT_ADDR,
            1_000_000u128
        )]
    );

    // We verify the balance of the token is the right one, with the tokens deposited
    let new_balance: BalanceResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            cw20_base::msg::QueryMsg::Balance {
                address: "depositor".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(new_balance.balance, Uint128::from(1_000_000u128));
}

#[test]
fn test_deposit_funds_share() {
    let mut deps = init_env(Some("0.1"));
    deps.querier.with_bond_share(1000000, 4000000);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("depositor", &[]),
        ExecuteMsg::MintWith {
            recipient: "depositor".to_string(),
            lsd_amount: 1_000_000u128.into(),
        },
    )
    .unwrap();
    // We assert the messages transfer the underyling asset
    assert_eq!(
        res.messages,
        vec![get_transfer_from_msg(
            MOCK_SPECTRUM_TOKEN,
            "depositor",
            MOCK_CONTRACT_ADDR,
            1_000_000u128
        )]
    );

    // We verify the balance of the token is the right one, with the tokens deposited
    let new_balance: BalanceResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            cw20_base::msg::QueryMsg::Balance {
                address: "depositor".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(new_balance.balance, Uint128::from(4_000_000u128));
}

// What happens when we need to decompound (exchange rate changes)
#[test]
fn test_decompound_no_limit() {
    let mut deps = init_env(None);

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("depositor", &[]),
        ExecuteMsg::MintWith {
            recipient: "depositor".to_string(),
            lsd_amount: 1_000_000u128.into(),
        },
    )
    .unwrap();

    deps.querier.with_token_balances(&[(
        &MOCK_SPECTRUM_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(1_000_000u128),
        )],
    )]);
    deps.querier.with_bond_share(1000000, 4000000);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(1);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap();

    // OLD value : 1_000_000 of lsd deposited at 1.vs.1 exchange rate
    // NEW value : Now 1 unit of LSD equals 4 units of underlying asset.
    // SO the tokens are worth 4_000_000, only 250_000 tokens are needed to get a 1_000_000 value
    // 750_000 tokens need to be decompounded. Because we limit the compute errors, 1 is substracted

    assert_eq!(
        res.messages,
        vec![get_transfer_msg(
            MOCK_SPECTRUM_TOKEN,
            "hub",
            750_000u128 - 1u128
        )]
    );

    // We execute the transfer
    deps.querier.handle_execute(&res.messages).unwrap();

    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(2);
    // Decompounds 0 now
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap();

    assert_eq!(res.messages, vec![]);
}

// What happens when we need to decompound (exchange rate changes)
#[test]
fn test_decompound_same_time() {
    let mut deps = init_env(Some("0.1"));

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("depositor", &[]),
        ExecuteMsg::MintWith {
            recipient: "depositor".to_string(),
            lsd_amount: 1_000_000u128.into(),
        },
    )
    .unwrap();

    deps.querier.with_token_balances(&[(
        &MOCK_SPECTRUM_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(1_000_000u128),
        )],
    )]);
    deps.querier.with_bond_share(1000000, 4000000);

    // Can't execute decompound with same block time as before
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Std(StdError::generic_err("Can't decompound too often",)));
}

// What happens when we need to decompound (exchange rate changes)
#[test]
fn test_decompound_with_limit() {
    let mut deps = init_env(Some("0.1"));

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("depositor", &[]),
        ExecuteMsg::MintWith {
            recipient: "depositor".to_string(),
            lsd_amount: 1_000_000u128.into(),
        },
    )
    .unwrap();

    deps.querier.with_token_balances(&[(
        &MOCK_SPECTRUM_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(1_000_000u128),
        )],
    )]);
    deps.querier.with_bond_share(1000000, 4000000);

    
    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(3600*24);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap();

    // OLD value : 1_000_000 of lsd deposited at 1.vs.1 exchange rate
    // NEW value : Now 1 unit of LSD equals 4 units of underlying asset.
    // BUT we limit the decompounding to 10%/year
    // So only 27 tokens may be decompounded per day
    // Always the -1 to account for computation errors

    assert_eq!(
        res.messages,
        vec![get_transfer_msg(
            MOCK_SPECTRUM_TOKEN,
            "hub",
            273u128 - 1u128
        )]
    );

    // We execute the transfer
    deps.querier.handle_execute(&res.messages).unwrap();

    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(2*3600*24);
    // Decompounds 0 now
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![get_transfer_msg(
            MOCK_SPECTRUM_TOKEN,
            "hub",
            275u128 - 1u128
        )]
    );

    // We execute the transfer
    deps.querier.handle_execute(&res.messages).unwrap();

    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(3*3600*24);
    // Decompounds 0 now
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("hub", &[]),
        ExecuteMsg::Decompound { recipient: None },
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![get_transfer_msg(
            MOCK_SPECTRUM_TOKEN,
            "hub",
            274u128 - 1u128
        )]
    );
}