use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use cw_ownable::cw_ownable_execute;

use crate::state::Launch;

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
        whitelist_max_buy: Option<u16>,
        whitelist_started_at: Uint64,
        whitelist_ended_at: Uint64,
        public_price: Uint128,
        public_max_buy: Option<u16>,
        public_started_at: Uint64,
        public_ended_at: Uint64,
        price_denom: String,
        royalty_percentage: Option<u64>,
        royalty_payment_address: Option<String>,
    },
    RemoveLaunch {
        contract_address: String,
    },
    ModifyLaunch {
        contract_address: String,
        max_supply: Option<u64>,
        base_uri: Option<String>,
        is_base_uri_static: Option<bool>,
        media_extension: Option<String>,
        whitelist_price: Option<Uint128>,
        whitelist_max_buy: Option<u16>,
        whitelist_started_at: Option<Uint64>,
        whitelist_ended_at: Option<Uint64>,
        public_price: Option<Uint128>,
        public_max_buy: Option<u16>,
        public_started_at: Option<Uint64>,
        public_ended_at: Option<Uint64>,
        price_denom: Option<String>,
    },
    AddToWhitelist {
        contract_address: String,
        account_addresses: Vec<String>,
    },
    RemoveToWhitelist {
        contract_address: String,
        account_addresses: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Launch)]
    GetLaunch { contract_address: String },
    #[returns(LaunchStatus)]
    GetLaunchStatus { contract_address: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetLaunch {
    pub count: i32,
}

#[cw_serde]
pub struct LaunchStatus {
    pub status: String,
}
