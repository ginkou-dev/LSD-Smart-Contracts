use std::marker::PhantomData;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::Decimal256;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};

use basset::external::{LSDQueryMsg, LSDStateResponse};
use basset::oracle::PriceResponse;
use basset::oracle::QueryMsg as OracleQueryMsg;

pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";

pub const MOCK_LSD_HUB_CONTRACT_ADDR: &str = "lsd";
pub const MOCK_LSD_TOKEN_CONTRACT_ADDR: &str = "lsd_token";

pub const MOCK_LSD_DENOM: &str = "stluna";
pub const MOCK_LSD_UNDERLYING_DENOM: &str = "uluna";

pub const MOCK_ORACLE_CONTRACT_ADDR: &str = "oracle";

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
    base: MockQuerier<Empty>,
    lsd_state_querier: LsdStateQuerier,
    oracle_price_querier: OraclePriceQuerier,
}

#[derive(Clone)]
pub struct LsdStateQuerier {
    // this lets us iterate over all pairs that match the first string
    lsd_state: Option<LSDStateResponse>,
}

impl LsdStateQuerier {
    pub fn new(lsd_state: LSDStateResponse) -> Self {
        LsdStateQuerier {
            lsd_state: Some(lsd_state),
        }
    }
}

#[derive(Clone)]
pub struct OraclePriceQuerier {
    // this lets us iterate over all pairs that match the first string
    price: Option<Decimal256>,
}

impl OraclePriceQuerier {
    pub fn new(price: Decimal256) -> Self {
        OraclePriceQuerier { price: Some(price) }
    }
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
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if *contract_addr == MOCK_LSD_HUB_CONTRACT_ADDR {
                    let lsd_message: LSDQueryMsg = from_binary(msg).unwrap();
                    match lsd_message {
                        LSDQueryMsg::State {} => {
                            let state_response = self.lsd_state_querier.lsd_state.clone().unwrap();
                            SystemResult::Ok(ContractResult::from(to_binary(&state_response)))
                        }
                    }
                } else if *contract_addr == MOCK_ORACLE_CONTRACT_ADDR {
                    let oracle_message: OracleQueryMsg = from_binary(msg).unwrap();
                    match oracle_message {
                        OracleQueryMsg::Price { base, quote } => {
                            if base != MOCK_LSD_DENOM || quote != MOCK_LSD_UNDERLYING_DENOM {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: "interpreting tokens".to_string(),
                                    request: msg.clone(),
                                });
                            }
                            let quote_response = PriceResponse {
                                rate: self.oracle_price_querier.price.unwrap(),
                                last_updated_base: 0,
                                last_updated_quote: 0,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&quote_response)))
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    unimplemented!()
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
            lsd_state_querier: LsdStateQuerier { lsd_state: None },
            oracle_price_querier: OraclePriceQuerier { price: None },
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_lsd_state(&mut self, lsd_state: LSDStateResponse) {
        self.lsd_state_querier = LsdStateQuerier::new(lsd_state);
    }

    // configure the mint whitelist mock querier
    pub fn with_oracle_price(&mut self, price: Decimal256) {
        self.oracle_price_querier = OraclePriceQuerier::new(price);
    }
}
