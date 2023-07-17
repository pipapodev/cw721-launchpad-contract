use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    pub taker_fee: Uint64,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        contract_address: String,
        receiver_address: Option<String>,
    },
    AddLaunch {
        owner_address: String,
        contract_address: String,
        max_supply: u64,
        base_uri: String,
        is_base_uri_static: bool,
        media_extension: Option<String>,
        whitelist_price: Uint128,
        whitelist_max_buy: u16,
        whitelist_started_at: Uint64,
        whitelist_ended_at: Uint64,
        public_price: Uint128,
        public_max_buy: u16,
        public_started_at: Uint64,
        public_ended_at: Uint64,
        price_denom: String,
        royalty_percentage: Option<u64>,
        royalty_payment_address: Option<String>,
    },
    RemoveLaunch {
        contract_address: String,
    },
    ModifyLaunch {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    // GetLaunch { contract_address: String },
    // GetLaunchStatus {
    //     contract_address: String,
    // },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetLaunch {
    pub count: i32,
}
