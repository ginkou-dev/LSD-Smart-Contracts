use wrapper_implementations::steak::SteakInitMsg;

use cw20_base::msg::{ExecuteMsg, QueryMsg};

use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: SteakInitMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    };
}

