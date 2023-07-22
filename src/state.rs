use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Launch {
    pub owner_address: Addr,
    pub max_supply: u64,
    pub base_uri: String,
    pub is_base_uri_static: bool,
    pub media_extension: Option<String>,
    pub whitelist_price: Coin,
    pub whitelist_max_buy: Option<u16>,
    pub whitelist_started_at: u64,
    pub whitelist_ended_at: u64,
    pub public_price: Coin,
    pub public_max_buy: Option<u16>,
    pub public_started_at: u64,
    pub public_ended_at: u64,
    pub last_token_id: u64,
    pub royalty_percentage: Option<u64>,
    // https://github.com/CosmWasm/cw-nfts/blob/main/contracts/cw2981-royalties/src/lib.rs#L45
    pub royalty_payment_address: Option<String>,
}

pub type ContractAddress = Addr;

pub const TAKERFEE: Item<u64> = Item::new("taker_fee");
pub const LAUNCHES: Map<&Addr, Launch> = Map::new("launches");
