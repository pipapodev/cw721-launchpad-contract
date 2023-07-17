#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::TAKERFEE;
use cw721_rewards::{helpers::Cw721Contract, msg::ExecuteMsg as Cw721ExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-launchpad-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&info.sender.to_string()))?;

    TAKERFEE.save(deps.storage, &msg.taker_fee.u64())?;
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
        } => execute::mint(deps, env, info, contract_address, receiver_address),
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
            price_denom,
            royalty_percentage,
            royalty_payment_address,
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
            price_denom,
            royalty_percentage,
            royalty_payment_address,
        ),
        ExecuteMsg::RemoveLaunch { contract_address } => {
            execute::remove_launch(deps, info, contract_address)
        }
        ExecuteMsg::ModifyLaunch {} => todo!(),
    }
}

pub mod execute {
    use std::marker::PhantomData;

    use cosmwasm_std::{Empty, StdError, Uint128, Uint64};
    use cw721_rewards::Metadata;

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
    ) -> Result<Response, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        let owner_address = deps.api.addr_validate(&owner_address)?;
        let contract_address = deps.api.addr_validate(&contract_address)?;

        LAUNCHES.save(
            deps.storage,
            &contract_address,
            &Launch {
                owner_address,
                max_supply,
                base_uri,
                is_base_uri_static,
                media_extension,
                whitelist_price: whitelist_price.u128(),
                whitelist_max_buy,
                whitelist_started_at: whitelist_started_at.u64(),
                whitelist_ended_at: whitelist_ended_at.u64(),
                public_price: public_price.u128(),
                public_max_buy,
                public_started_at: public_started_at.u64(),
                public_ended_at: public_ended_at.u64(),
                price_denom,
                last_token_id: 0,
                royalty_percentage,
                royalty_payment_address,
            },
        )?;

        Ok(Response::new().add_attribute("action", "add_launch"))
    }

    pub fn remove_launch(
        deps: DepsMut,
        info: MessageInfo,
        contract_address: String,
    ) -> Result<Response, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
        let contract_address = deps.api.addr_validate(&contract_address)?;

        LAUNCHES.remove(deps.storage, &contract_address);

        Ok(Response::new().add_attribute("action", "remove_launch"))
    }

    pub fn mint(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract_address: String,
        receiver_id: Option<String>,
    ) -> Result<Response, ContractError> {
        let contract_address = deps.api.addr_validate(&contract_address)?;
        let mut launch = LAUNCHES.load(deps.storage, &contract_address)?;

        // check funds
        let fund_input = cw_utils::must_pay(&info, &launch.price_denom).unwrap();

        // TODO : determine minting status
        if fund_input.u128() != launch.public_price {
            return Err(ContractError::InsufficientFunds {});
        }

        // check if last_token_id < total_supply
        if launch.last_token_id >= launch.max_supply {
            return Err(ContractError::SoldOut {});
        }

        // TODO: determine minting status, public only for now
        let current_timestamp_in_seconds = env.block.time.seconds();
        if launch.public_started_at > current_timestamp_in_seconds
            || launch.public_ended_at < current_timestamp_in_seconds
        {
            return Err(ContractError::Closed {});
        }

        let receiver_id = if let Some(receiver_id) = receiver_id {
            deps.api.addr_validate(receiver_id.as_str())?
        } else {
            info.sender
        };

        // prepare call
        let token_id = (launch.last_token_id + 1).to_string();
        launch.last_token_id += 1;

        let media_extension = if let Some(media_extension) = launch.media_extension.clone() {
            media_extension
        } else {
            "png".to_string()
        };

        let mint_msg = Cw721ExecuteMsg::<Option<Metadata>>::Mint {
            token_id: token_id,
            owner: receiver_id.to_string(),
            token_uri: Some(format!(
                "{}/{}.{}",
                launch.base_uri, token_id, media_extension
            )),
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

        let callback = Cw721Contract::<Empty, Empty>(contract_address, PhantomData, PhantomData)
            .call(mint_msg)?;

        Ok(Response::new().add_message(callback))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

pub mod query {
    use super::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Uint64};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            taker_fee: Uint64::new(10),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg {};
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg {};
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
