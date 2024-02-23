use cosmwasm_schema::write_api;

use wrapper_implementations::coin::StrideInitMsg;
use cw20_base::msg::QueryMsg;
use basset::wrapper::ExecuteMsg;
fn main() {
    write_api! {
        instantiate: StrideInitMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    };
}
