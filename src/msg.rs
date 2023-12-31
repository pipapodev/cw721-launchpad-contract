use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint64};
use cw_ownable::cw_ownable_execute;

#[allow(unused_imports)]
use crate::state::Launch;

#[cw_serde]
pub struct InstantiateMsg {
    pub taker_fee: Uint64,
    pub native_denom: String,
    pub taker_address: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    ChangeTakerFee {
        taker_fee: Uint64,
    },
    Mint {
        contract_address: String,
        receiver_address: Option<String>,
        proof: Option<Vec<String>>,
    },
    AddLaunch {
        owner_address: String,
        contract_address: String,
        max_supply: u64,
        base_uri: String,
        is_base_uri_static: bool,
        media_extension: Option<String>,
        whitelist_price: Coin,
        whitelist_max_buy: Option<u16>,
        whitelist_started_at: Uint64,
        whitelist_ended_at: Uint64,
        public_price: Coin,
        public_max_buy: Option<u16>,
        public_started_at: Uint64,
        public_ended_at: Uint64,
        royalty_percentage: Option<u64>,
        royalty_payment_address: Option<String>,
        whitelist_merkle_root: Option<String>,
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
        whitelist_price: Option<Coin>,
        whitelist_max_buy: Option<u16>,
        whitelist_started_at: Option<Uint64>,
        whitelist_ended_at: Option<Uint64>,
        public_price: Option<Coin>,
        public_max_buy: Option<u16>,
        public_started_at: Option<Uint64>,
        public_ended_at: Option<Uint64>,
        whitelist_merkle_root: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Launch)]
    GetLaunch { contract_address: String },
    #[returns(LaunchStatus)]
    GetLaunchStatus { contract_address: String },
    #[returns(WhitelistStatus)]
    GetWhitelistStatus {
        contract_address: String,
        account_address: String,
        proof: Vec<String>,
    },
}

#[cw_serde]
pub struct LaunchStatus {
    pub status: String,
}

#[cw_serde]
pub struct WhitelistStatus {
    pub is_whitelist: bool,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
