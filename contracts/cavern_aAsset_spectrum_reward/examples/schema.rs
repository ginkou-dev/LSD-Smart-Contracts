use basset::reward::{ InstantiateMsg, QueryMsg, ExecuteMsg};

use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    };
}

