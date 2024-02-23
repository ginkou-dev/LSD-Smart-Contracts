use std::str::FromStr;
use cosmwasm_std::Decimal;
use basset::eris_lp::StateResponse;
use astroport::asset::PairInfo;
use basset::eris_lp::QueryMsg;

use basset::dex_router::{
    AssetInfo, QueryMsg as SwapQueryMsg, SimulateSwapOperationsResponse, SwapOperation,
};

use cosmwasm_std::from_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::CanonicalAddr;
use cosmwasm_std::Uint128;
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;

use basset::hub::Config;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::Empty;
use cosmwasm_std::{
    from_slice, to_binary, Api, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::marker::PhantomData;
use cw20::BalanceResponse as Cw20BalanceResponse;

pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";
pub const MOCK_REWARD_CONTRACT_ADDR: &str = "reward";
pub const MOCK_TOKEN_CONTRACT_ADDR: &str = "token";

pub const MOCK_AMP_LP_TOKEN: &str = "amp_lp_token";
pub const MOCK_AMP_HUB: &str = "amp_hub_token";
pub const MOCK_ASTROPORT_PAIR: &str = "astroport_pair";
pub const MOCK_ASTROPORT_LP_TOKEN: &str = "astroport_lp_token";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = String::from(MOCK_CONTRACT_ADDR);
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(&contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    pub base: MockQuerier<Empty>,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                if *contract_addr == MOCK_HUB_CONTRACT_ADDR {
                    let prefix_config = to_length_prefixed(b"config").to_vec();
                    let api: MockApi = MockApi::default();
                    if key.as_slice().to_vec() == prefix_config {
                        let config = Config {
                            creator: api.addr_validate(&String::from("owner1")).unwrap(),
                            reward_contract: Some(
                                api.addr_validate(&String::from(MOCK_REWARD_CONTRACT_ADDR))
                                    .unwrap(),
                            ),
                            token_contract: Some(
                                api.addr_validate(&String::from(MOCK_TOKEN_CONTRACT_ADDR))
                                    .unwrap(),
                            ),
                        };
                        SystemResult::Ok(ContractResult::from(to_binary(&config)))
                    } else {
                        unimplemented!();
                    }
                } else {
                    let key: &[u8] = key.as_slice();

                    let _prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                    let prefix_balance = to_length_prefixed(b"balance").to_vec();

                    let balances: &HashMap<String, Uint128> =
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => balances,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!(
                                        "No balance info exists for the contract {}",
                                        contract_addr
                                    ),
                                    request: key.into(),
                                })
                            }
                        };

                    if key[..prefix_balance.len()].to_vec() == prefix_balance {
                        let key_address: &[u8] = &key[prefix_balance.len()..];
                        let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
                        let api: MockApi = MockApi::default();
                        let address: Addr = match api.addr_humanize(&address_raw) {
                            Ok(v) => v,
                            Err(e) => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!("Parsing query request: {}", e),
                                    request: key.into(),
                                })
                            }
                        };
                        let balance = match balances.get(&address.to_string()) {
                            Some(v) => v,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: "Balance not found".to_string(),
                                    request: key.into(),
                                })
                            }
                        };
                        SystemResult::Ok(ContractResult::from(to_binary(
                            &to_binary(&balance).unwrap(),
                        )))
                    } else {
                        unimplemented!()
                    }
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if *contract_addr == "astroport_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::AstroSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount * Uint128::from(9u128)
                                        / Uint128::from(10u128),
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else if *contract_addr == "phoenix_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::TokenSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount * Uint128::from(11u128)
                                        / Uint128::from(10u128),
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else if *contract_addr == "terraswap_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::TerraSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount,
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else if *contract_addr == MOCK_AMP_HUB{
                    match from_binary(msg).unwrap(){
                        QueryMsg::State{
                            addr: None
                        } => {
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &StateResponse {
                                    total_lp: 1000000u128.into(),
                                    total_amp_lp: 4000000u128.into(),
                                    exchange_rate: Decimal::from_str("4").unwrap(),
                                    pair_contract: Addr::unchecked("pair"),
                                    locked_assets: vec![],
                                    user_info: None
                                },
                            )))
                        },
                        _=> {
                            unimplemented!()
                        }
                    }
                } else if *contract_addr == MOCK_ASTROPORT_PAIR{
                    match from_binary(msg).unwrap(){
                        astroport::pair::QueryMsg::Pair {  } => SystemResult::Ok(ContractResult::from(to_binary(
                                &PairInfo {
                                    liquidity_token: Addr::unchecked(MOCK_ASTROPORT_LP_TOKEN),
                                    asset_infos: vec![],
                                    contract_addr: Addr::unchecked(contract_addr),
                                    pair_type: astroport::factory::PairType::Xyk {  }
                                },
                            ))),
                        _ => unimplemented!()
                    }
                } else {
                     // If we don't recognize it, we simply assume it's a cw20 query
                    match from_binary(msg).unwrap() {
                        Cw20QueryMsg::TokenInfo {} => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };
                            let mut total_supply = Uint128::zero();

                            for balance in balances {
                                total_supply += *balance.1;
                            }
                            let _api: MockApi = MockApi::default();
                            let token_inf: TokenInfoResponse = TokenInfoResponse {
                                name: "bluna".to_string(),
                                symbol: "BLUNA".to_string(),
                                decimals: 6,
                                total_supply,
                            };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&token_inf).unwrap()))
                        }
                        Cw20QueryMsg::Balance { address } => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };

                            let balance = match balances.get(&address) {
                                Some(v) => *v,
                                None => {
                                    return SystemResult::Ok(ContractResult::Ok(
                                        to_binary(&Cw20BalanceResponse {
                                            balance: Uint128::zero(),
                                        })
                                        .unwrap(),
                                    ));
                                }
                            };

                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                            ))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }

                }
            }
            _ => self.base.handle_query(request),
        }
    }
}
impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
