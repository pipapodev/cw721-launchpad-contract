#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{NATIVE_DENOM, TAKERADDRESS, TAKERFEE};
use cw721_rewards::{helpers::Cw721Contract, msg::ExecuteMsg as Cw721ExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-launchpad-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
use sha2::Digest;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&info.sender.to_string()))?;

    TAKERFEE.save(deps.storage, &msg.taker_fee.u64())?;
    NATIVE_DENOM.save(deps.storage, &msg.native_denom)?;
    TAKERADDRESS.save(
        deps.storage,
        &deps.api.addr_validate(&msg.taker_address).unwrap(),
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => execute::update_ownership(deps, env, info, action),
        ExecuteMsg::Mint {
            contract_address,
            receiver_address,
            proof,
        } => execute::mint(deps, env, info, contract_address, receiver_address, proof),
        ExecuteMsg::AddLaunch {
            owner_address,
            contract_address,
            max_supply,
            base_uri,
            is_base_uri_static,
            media_extension,
            whitelist_price,
            whitelist_max_buy,
            whitelist_started_at,
            whitelist_ended_at,
            public_price,
            public_max_buy,
            public_started_at,
            public_ended_at,
            royalty_percentage,
            royalty_payment_address,
            whitelist_merkle_root,
        } => execute::add_launch(
            deps,
            env,
            info,
            owner_address,
            contract_address,
            max_supply,
            base_uri,
            is_base_uri_static,
            media_extension,
            whitelist_price,
            whitelist_max_buy,
            whitelist_started_at,
            whitelist_ended_at,
            public_price,
            public_max_buy,
            public_started_at,
            public_ended_at,
            royalty_percentage,
            royalty_payment_address,
            whitelist_merkle_root,
        ),
        ExecuteMsg::RemoveLaunch { contract_address } => {
            execute::remove_launch(deps, info, contract_address)
        }
        ExecuteMsg::ModifyLaunch {
            contract_address,
            max_supply,
            base_uri,
            is_base_uri_static,
            media_extension,
            whitelist_price,
            whitelist_max_buy,
            whitelist_started_at,
            whitelist_ended_at,
            public_price,
            public_max_buy,
            public_started_at,
            public_ended_at,
            whitelist_merkle_root,
        } => execute::modify_launch(
            deps,
            env,
            info,
            contract_address,
            max_supply,
            base_uri,
            is_base_uri_static,
            media_extension,
            whitelist_price,
            whitelist_max_buy,
            whitelist_started_at,
            whitelist_ended_at,
            public_price,
            public_max_buy,
            public_started_at,
            public_ended_at,
            whitelist_merkle_root,
        ),
        ExecuteMsg::ChangeTakerFee { taker_fee } => {
            execute::change_taker_fee(deps, info, taker_fee)
        }
    }
}

pub mod execute {
    use std::marker::PhantomData;

    use cosmwasm_std::{coins, Addr, BankMsg, Coin, Decimal, Empty, Uint128, Uint64};
    use cw721_rewards::Metadata;
    use cw_storage_plus::Map;

    use crate::state::{Launch, LAUNCHES};

    use super::*;

    pub fn update_ownership(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: cw_ownable::Action,
    ) -> Result<Response, ContractError> {
        let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
        Ok(Response::new().add_attributes(ownership.into_attributes()))
    }

    pub fn add_launch(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
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
    ) -> Result<Response, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        let owner_address = deps.api.addr_validate(&owner_address)?;
        let contract_address = deps.api.addr_validate(&contract_address)?;

        let exist = LAUNCHES.load(deps.storage, &contract_address);

        if exist.is_ok() {
            return Err(ContractError::LaunchAlreadyExist {});
        }

        let native_denom = NATIVE_DENOM.load(deps.storage)?;
        if whitelist_price.denom != native_denom {
            return Err(ContractError::DenomNotSupported {});
        }
        if public_price.denom != native_denom {
            return Err(ContractError::DenomNotSupported {});
        }

        LAUNCHES.save(
            deps.storage,
            &contract_address,
            &Launch {
                owner_address,
                max_supply,
                base_uri,
                is_base_uri_static,
                media_extension,
                whitelist_price: whitelist_price,
                whitelist_max_buy,
                whitelist_started_at: whitelist_started_at.u64(),
                whitelist_ended_at: whitelist_ended_at.u64(),
                public_price: public_price,
                public_max_buy,
                public_started_at: public_started_at.u64(),
                public_ended_at: public_ended_at.u64(),
                last_token_id: 0,
                royalty_percentage,
                royalty_payment_address,
                whitelist_merkle_root,
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "add_launch")
            .add_attribute("contract_address", contract_address))
    }

    pub fn modify_launch(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
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
    ) -> Result<Response, ContractError> {
        let contract_address = deps.api.addr_validate(&contract_address)?;

        let launch = LAUNCHES.load(deps.storage, &contract_address)?;

        let is_owner = cw_ownable::assert_owner(deps.storage, &info.sender);

        if is_owner.is_err() {
            if info.sender != launch.owner_address {
                return Err(ContractError::Unauthorized {});
            }
        }

        let native_denom = NATIVE_DENOM.load(deps.storage)?;
        if let Some(ref whitelist_price) = whitelist_price {
            if whitelist_price.denom != native_denom {
                return Err(ContractError::DenomNotSupported {});
            }
        }
        if let Some(ref public_price) = public_price {
            if public_price.denom != native_denom {
                return Err(ContractError::DenomNotSupported {});
            }
        }

        LAUNCHES.save(
            deps.storage,
            &contract_address,
            &Launch {
                owner_address: launch.owner_address,
                max_supply: if let Some(max_supply) = max_supply {
                    max_supply
                } else {
                    launch.max_supply
                },
                base_uri: if let Some(base_uri) = base_uri {
                    base_uri
                } else {
                    launch.base_uri
                },
                is_base_uri_static: if let Some(is_base_uri_static) = is_base_uri_static {
                    is_base_uri_static
                } else {
                    launch.is_base_uri_static
                },
                media_extension: if let Some(media_extension) = media_extension {
                    Some(media_extension)
                } else {
                    launch.media_extension
                },
                whitelist_price: if let Some(whitelist_price) = whitelist_price {
                    whitelist_price
                } else {
                    launch.whitelist_price
                },
                whitelist_max_buy: if let Some(whitelist_max_buy) = whitelist_max_buy {
                    Some(whitelist_max_buy)
                } else {
                    launch.whitelist_max_buy
                },
                whitelist_started_at: if let Some(whitelist_started_at) = whitelist_started_at {
                    whitelist_started_at.u64()
                } else {
                    launch.whitelist_started_at
                },
                whitelist_ended_at: if let Some(whitelist_ended_at) = whitelist_ended_at {
                    whitelist_ended_at.u64()
                } else {
                    launch.whitelist_ended_at
                },
                public_price: if let Some(public_price) = public_price {
                    public_price
                } else {
                    launch.public_price
                },
                public_max_buy: if let Some(public_max_buy) = public_max_buy {
                    Some(public_max_buy)
                } else {
                    launch.public_max_buy
                },
                public_started_at: if let Some(public_started_at) = public_started_at {
                    public_started_at.u64()
                } else {
                    launch.public_started_at
                },
                public_ended_at: if let Some(public_ended_at) = public_ended_at {
                    public_ended_at.u64()
                } else {
                    launch.public_ended_at
                },
                last_token_id: launch.last_token_id,
                royalty_percentage: launch.royalty_percentage,
                royalty_payment_address: launch.royalty_payment_address,
                whitelist_merkle_root: if let Some(whitelist_merkle_root) = whitelist_merkle_root {
                    Some(whitelist_merkle_root)
                } else {
                    launch.whitelist_merkle_root
                },
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "modify_launch")
            .add_attribute("contract_address", contract_address))
    }

    pub fn remove_launch(
        deps: DepsMut,
        info: MessageInfo,
        contract_address: String,
    ) -> Result<Response, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
        let contract_address = deps.api.addr_validate(&contract_address)?;

        LAUNCHES.remove(deps.storage, &contract_address);

        Ok(Response::new()
            .add_attribute("action", "remove_launch")
            .add_attribute("contract_address", contract_address))
    }

    pub fn mint(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract_address: String,
        receiver_address: Option<String>,
        proof: Option<Vec<String>>,
    ) -> Result<Response, ContractError> {
        let contract_address = deps.api.addr_validate(&contract_address)?;
        let mut launch = LAUNCHES.load(deps.storage, &contract_address)?;

        // check funds
        let fund_input;

        // check if last_token_id < total_supply
        if launch.last_token_id >= launch.max_supply {
            return Err(ContractError::SoldOut {});
        }

        let account_address = info.sender.clone();
        let receiver_address = if let Some(receiver_id) = receiver_address {
            deps.api.addr_validate(receiver_id.as_str())?
        } else {
            account_address.clone()
        };

        // Determine minting status
        let denom;
        let price: Uint128;
        let current_timestamp_in_seconds = env.block.time.seconds();
        if current_timestamp_in_seconds > launch.whitelist_started_at
            && current_timestamp_in_seconds < launch.whitelist_ended_at
        {
            // WHITELIST MINT
            denom = &launch.whitelist_price.denom;
            price = launch.whitelist_price.amount;

            fund_input = cw_utils::must_pay(&info, denom).unwrap();
            // check if user in whitelist

            let is_whitelisted = verify_merkle_proof(
                account_address.to_string(),
                launch.whitelist_merkle_root.as_ref().unwrap(),
                proof.unwrap(),
            );

            if !is_whitelisted {
                return Err(ContractError::NotWhitelisted {});
            }

            // whitelist
            if fund_input.u128() < launch.whitelist_price.amount.u128() {
                return Err(ContractError::InsufficientFunds {});
            }

            // check whitelist quota
            if let Some(whitelist_max_buy) = launch.whitelist_max_buy {
                let whitelist_items_key = format!("{}-{}", contract_address, "whitelistitems");
                let whitelist_items: Map<&Addr, u64> = Map::new(whitelist_items_key.as_str());

                let previous_items = whitelist_items
                    .load(deps.storage, &receiver_address)
                    .unwrap_or(0);

                if previous_items >= whitelist_max_buy as u64 {
                    return Err(ContractError::MintQuotaExhausted {});
                }
                whitelist_items.save(deps.storage, &receiver_address, &(previous_items + 1))?;
            }
        } else if current_timestamp_in_seconds > launch.public_started_at
            && current_timestamp_in_seconds < launch.public_ended_at
        {
            denom = &launch.public_price.denom;
            fund_input = cw_utils::must_pay(&info, denom).unwrap();
            price = launch.public_price.amount;

            // public
            if fund_input.u128() < launch.public_price.amount.u128() {
                return Err(ContractError::InsufficientFunds {});
            }

            // check public quota
            if let Some(public_max_buy) = launch.public_max_buy {
                let public_items_key = format!("{}-{}", contract_address, "publicitems");
                let public_items: Map<&Addr, u64> = Map::new(public_items_key.as_str());

                let previous_items = public_items
                    .load(deps.storage, &receiver_address)
                    .unwrap_or(0);

                if previous_items >= public_max_buy as u64 {
                    return Err(ContractError::MintQuotaExhausted {});
                }
                public_items.save(deps.storage, &receiver_address, &(previous_items + 1))?;
            }
        } else {
            return Err(ContractError::Closed {});
        }

        // prepare call
        let token_id = (launch.last_token_id + 1).to_string();
        launch.last_token_id += 1;

        let token_uri = if launch.is_base_uri_static {
            launch.base_uri.clone()
        } else {
            let media_extension = if let Some(media_extension) = launch.media_extension.clone() {
                media_extension
            } else {
                "png".to_string()
            };

            format!("{}/{}.{}", launch.base_uri, token_id, media_extension)
        };

        let mint_msg = Cw721ExecuteMsg::<Option<Metadata>>::Mint {
            token_id: token_id.clone(),
            owner: receiver_address.to_string(),
            token_uri: Some(token_uri),
            extension: Some(Metadata {
                royalty_percentage: launch.royalty_percentage.clone(),
                royalty_payment_address: launch.royalty_payment_address.clone(),
                image: None,
                image_data: None,
                external_url: None,
                description: None,
                name: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
            }),
        };

        LAUNCHES.save(deps.storage, &contract_address, &launch)?;

        let callback =
            Cw721Contract::<Empty, Empty>(contract_address.clone(), PhantomData, PhantomData)
                .call(mint_msg)?;

        let mut messages = Vec::new();
        messages.push(callback);

        // fund transfers
        // marketplace funds
        let taker_fee = TAKERFEE.load(deps.storage)?;
        let taker_funds = fund_input * Decimal::percent(taker_fee);

        if taker_funds.u128() > 0 {
            let send_taker_funds_msg = BankMsg::Send {
                to_address: TAKERADDRESS.load(deps.storage).unwrap().to_string(),
                amount: coins(taker_funds.u128(), denom),
            };
            messages.push(send_taker_funds_msg.into())
        }

        // project owner funds
        let owner_funds = fund_input - taker_funds;
        if owner_funds.u128() > 0 {
            let send_owner_funds_msg = BankMsg::Send {
                to_address: launch.owner_address.to_string(),
                amount: coins(owner_funds.u128(), denom),
            };

            messages.push(send_owner_funds_msg.into())
        }

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "mint_from_launchpad")
            .add_attribute("contract_address", contract_address.to_string())
            .add_attribute("token_id", token_id)
            .add_attribute("receiver_address", receiver_address)
            .add_attribute("price", price))
    }

    pub fn change_taker_fee(
        deps: DepsMut,
        info: MessageInfo,
        taker_fee: Uint64,
    ) -> Result<Response, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        TAKERFEE.save(deps.storage, &taker_fee.u64())?;

        Ok(Response::new()
            .add_attribute("action", "change_taker_fee")
            .add_attribute("taker_fee", taker_fee))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLaunch { contract_address } => {
            to_binary(&query::get_launch(deps, contract_address)?)
        }
        QueryMsg::GetLaunchStatus { contract_address } => {
            to_binary(&query::get_launch_status(deps, env, contract_address)?)
        }
        QueryMsg::GetWhitelistStatus {
            contract_address,
            account_address,
            proof,
        } => to_binary(&query::get_whitelist_status(
            deps,
            contract_address,
            account_address,
            proof,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub mod query {
    use crate::{
        msg::{LaunchStatus, WhitelistStatus},
        state::{Launch, LAUNCHES},
    };

    use super::*;

    pub fn get_launch(deps: Deps, contract_address: String) -> StdResult<Launch> {
        let launch = LAUNCHES.load(deps.storage, &deps.api.addr_validate(&contract_address)?)?;

        Ok(launch)
    }

    pub fn get_launch_status(
        deps: Deps,
        env: Env,
        contract_address: String,
    ) -> StdResult<LaunchStatus> {
        let launch = LAUNCHES.load(deps.storage, &deps.api.addr_validate(&contract_address)?)?;

        let current_timestamp_in_seconds = env.block.time.seconds();
        if current_timestamp_in_seconds > launch.whitelist_started_at
            && current_timestamp_in_seconds < launch.whitelist_ended_at
        {
            Ok(LaunchStatus {
                status: "whitelist".to_string(),
            })
        } else if current_timestamp_in_seconds > launch.public_started_at
            && current_timestamp_in_seconds < launch.public_ended_at
        {
            Ok(LaunchStatus {
                status: "public".to_string(),
            })
        } else {
            Ok(LaunchStatus {
                status: "closed".to_string(),
            })
        }
    }

    pub fn get_whitelist_status(
        deps: Deps,
        contract_address: String,
        account_address: String,
        proof: Vec<String>,
    ) -> StdResult<WhitelistStatus> {
        // verify merkle root
        let launch = get_launch(deps, contract_address).unwrap();

        let merkle_root = launch.whitelist_merkle_root.unwrap();

        if !verify_merkle_proof(account_address, &merkle_root, proof) {
            Ok(WhitelistStatus {
                is_whitelist: false,
            })
        } else {
            Ok(WhitelistStatus { is_whitelist: true })
        }
    }
}

fn verify_merkle_proof(user_input: String, merkle_root: &String, proof: Vec<String>) -> bool {
    // https://github.com/CosmWasm/cw-tokens/blob/main/contracts/cw20-merkle-airdrop/src/contract.rs#L282
    let hash = sha2::Sha256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .unwrap();

    let hash = proof
        .into_iter()
        .try_fold(hash, |hash, p| {
            let mut proof_buf = [0; 32];
            let _ = hex::decode_to_slice(p, &mut proof_buf);
            let mut hashes = [hash, proof_buf];
            hashes.sort_unstable();
            sha2::Sha256::digest(&hashes.concat()).as_slice().try_into()
        })
        .unwrap();

    let mut root_buf: [u8; 32] = [0; 32];
    let _ = hex::decode_to_slice(merkle_root, &mut root_buf);
    return root_buf == hash;
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Uint64};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            taker_fee: Uint64::new(10),
            native_denom: "aconst".to_string(),
            taker_address: "admin".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
