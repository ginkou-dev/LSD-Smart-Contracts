use cosmwasm_std::CosmosMsg;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::SubMsg;
use basset::external::{
    CTokenStateResponse, SpectrumQueryMsg, UserInfoResponse,
};
use basset::hub::Config;
use cosmwasm_std::WasmMsg;
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::Empty;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, AllBalanceResponse, Api, BalanceResponse, BankQuery,
    CanonicalAddr, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest,
    SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::Cw20ExecuteMsg;
use cw20::TokenInfoResponse;
use std::collections::HashMap;
use std::marker::PhantomData;

use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};

pub const MOCK_GENERATOR_ADDR: &str = "spectrum-generator";

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[]));

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
    bond_share_querier: BondShareQuerier,
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
                let prefix_config = to_length_prefixed(b"config").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();
                let api: MockApi = MockApi::default();

                if key.as_slice().to_vec() == prefix_config {
                    let config = Config {
                        creator: api.addr_validate("owner1").unwrap(),
                        reward_contract: Some(api.addr_validate("reward").unwrap()),
                        token_contract: Some(api.addr_validate("token").unwrap()),
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&config).unwrap(),
                    )))
                } else if key.as_slice()[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key.as_slice()[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
                    let balances: &HashMap<String, Uint128> =
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => balances,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!(
                                        "No balance info exists for the contract {}",
                                        contract_addr
                                    ),
                                    request: key.as_slice().into(),
                                })
                            }
                        };
                    let api: MockApi = MockApi::default();
                    let address: String = match api.addr_humanize(&address_raw) {
                        Ok(v) => v.to_string(),
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.as_slice().into(),
                            })
                        }
                    };
                    let balance = match balances.get(&address) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.as_slice().into(),
                            })
                        }
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&balance)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::AllBalances { address }) => {
                if address == &"reward".to_string() {
                    let mut coins: Vec<Coin> = vec![];
                    let luna = Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(luna);
                    let krt = Coin {
                        denom: "ukrt".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(krt);
                    let usd = Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(usd);
                    let all_balances = AllBalanceResponse { amount: coins };
                    SystemResult::Ok(ContractResult::from(to_binary(&all_balances)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::Balance { address, denom }) => {
                if address == &"reward".to_string() && denom == "uusd" {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: Uint128::new(2000u128),
                            denom: denom.to_string(),
                        },
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                println!("{:?}", contract_addr);
                // First we filter the query destination
                if contract_addr.as_str() == MOCK_GENERATOR_ADDR { match from_binary(msg).unwrap() {
                    SpectrumQueryMsg::State {} => {
                        // We always assume the spectrum state is the same for now
                        return SystemResult::Ok(ContractResult::from(to_binary(
                            &CTokenStateResponse {
                                total_bond_share: self.bond_share_querier.share,
                            },
                        )));
                    }
                    SpectrumQueryMsg::UserInfo { user: _, lp_token: _ } => {
                        return SystemResult::Ok(ContractResult::from(to_binary(
                            &UserInfoResponse {
                                bond_share: self.bond_share_querier.share,
                                bond_amount: self.bond_share_querier.amount,
                                reward_indexes: vec![],
                                pending_rewards: vec![],
                            },
                        )));
                    }
                    _ => panic!("Not unimplemented! for tests")
                } }

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
            _ => self.base.handle_query(request),
        }
    }

    // We handle the execute effects (especially token transfers)
    pub fn handle_execute(&mut self, all_msgs: &[SubMsg])-> StdResult<()>{
        // We execute the messages sequentially
        for msg in all_msgs{
            match msg.msg.clone(){
                CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds: _ }) => {
                    match from_binary(&msg)?{
                        Cw20ExecuteMsg::Transfer { recipient, amount } => {
                            self.token_querier.transfer(contract_addr, MOCK_CONTRACT_ADDR.to_string(), recipient, amount)
                        }
                        Cw20ExecuteMsg::TransferFrom { owner, recipient, amount } => {
                            self.token_querier.transfer(contract_addr, owner, recipient, amount)
                        },
                        _=> return Err(StdError::generic_err("Not handled"))
                    }
                },
                _ => return Err(StdError::generic_err("Not handled"))
            }
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }

    pub fn transfer(&mut self, token: String, from: String, to: String, value: Uint128){
        let token_balances = self.balances.get_mut(&token).unwrap();
        token_balances.insert(from.clone(), *token_balances.get(&from).unwrap() - value);
        token_balances.insert(to.clone(), *token_balances.get(&to).unwrap_or(&Uint128::zero()) + value);
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

#[derive(Clone, Default)]
pub struct BondShareQuerier {
    pub share: Uint128,
    pub amount: Uint128,
}

impl BondShareQuerier {
    pub fn new(share: Uint128, amount: Uint128) -> Self {
        BondShareQuerier { share, amount }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            bond_share_querier: BondShareQuerier::new(1u128.into(), 1u128.into()),
        }
    }

    // configure the mint whitelist mock basset
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    // configure the tax mock querier
    pub fn with_bond_share(&mut self, share: u128, amount: u128) {
        self.bond_share_querier = BondShareQuerier::new(share.into(), amount.into());
    }
}
