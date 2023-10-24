use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetAddressesBySocialResponse, GetSocialsByAddressResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{Platform, ProfileId, SocialInfo, UserInfo, ACCEPTED_TOKEN, OWNER, USER_INFOS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
const CONTRACT_NAME: &str = "cosmos:donex-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const FEE_PERCENTAGE: u128 = 5;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(deps.storage, &info.sender)?;
    ACCEPTED_TOKEN.save(deps.storage, &msg.accepted_token)?;
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
    use ExecuteMsg::*;
    match msg {
        SubmitSocial {
            social_info,
            address,
        } => submit_social_link(deps, info, social_info, address),
        Donate { recipient } => donate(deps, env, info, recipient),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddressesBySocial {
            profile_id,
            platform,
        } => to_binary(&query_by_social_link(deps, profile_id, platform)?),
        QueryMsg::GetSocialsByAddress { address } => to_binary(&query_by_address(deps, address)?),
    }
}

// submit link between social platform accounts and chain address
pub fn submit_social_link(
    deps: DepsMut,
    info: MessageInfo,
    social_info: SocialInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    // Check authorization
    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }
    let (platform_id, profile_id) = social_info;

    let user_info = UserInfo {
        address: address.clone(),
        platform_id: platform_id.clone(),
        profile_id: profile_id.clone(),
    };
    USER_INFOS.save(
        deps.storage,
        [address.to_string(), platform_id].join("_").as_str(),
        &user_info,
    )?;

    Ok(Response::new().add_attribute("method", "submit_social_link"))
}

pub fn donate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
) -> Result<Response, ContractError> {
    // Check if denom in accepted_token
    let accepted_tokens = ACCEPTED_TOKEN.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;
    // For now restricted to only 1 token per transaction
    // TODO: handle multiple token sent
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidDenom {});
    }
    let denom = info.funds.first().unwrap().denom.to_string();
    if !accepted_tokens.contains(&denom.to_string()) {
        return Err(ContractError::InvalidDenom {});
    }
    let donation = cw_utils::must_pay(&info, &denom)?.u128();
    let deducted_fee = donation * FEE_PERCENTAGE / 100;
    let recipent_amount = donation - deducted_fee;
    let message_recipent = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(recipent_amount, &denom),
    };
    let message_owner = BankMsg::Send {
        to_address: owner.to_string(),
        amount: coins(deducted_fee, &denom),
    };

    let resp = Response::new()
        .add_message(message_recipent)
        .add_message(message_owner)
        .add_attribute("action", "donate")
        .add_attribute("amount", recipent_amount.to_string())
        .add_attribute("sender", info.sender.to_string());

    Ok(resp)
}
fn query_by_social_link(
    deps: Deps,
    profile_id: ProfileId,
    platform: Platform,
) -> StdResult<GetAddressesBySocialResponse> {
    let social_info = (platform, profile_id);
    // Query by (platform, profile_id)
    let user_infos: Vec<_> = USER_INFOS
        .idx
        .social_info
        .prefix(social_info)
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();

    if user_infos.is_empty() {
        return Ok(GetAddressesBySocialResponse { address: vec![] });
    }
    let addresses = user_infos
        .iter()
        .map(|user_info| user_info.1.address.clone())
        .collect::<Vec<_>>();
    Ok(GetAddressesBySocialResponse { address: addresses })
}
fn query_by_address(deps: Deps, address: Addr) -> StdResult<GetSocialsByAddressResponse> {
    // Query by address
    let user_infos: Vec<_> = USER_INFOS
        .idx
        .address
        .prefix(address.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
        .collect();
    let social_infos = user_infos
        .iter()
        .map(|user_info| {
            (
                user_info.1.platform_id.clone(),
                user_info.1.profile_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    Ok(GetSocialsByAddressResponse { social_infos })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    #[test]
    fn submit_social() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    accepted_token: vec!["ucmst".to_string()],
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                addr,
                &ExecuteMsg::SubmitSocial {
                    social_info: ("twitter".to_string(), "123".to_string()),
                    address: Addr::unchecked("abc"),
                },
                &[],
            )
            .unwrap();
        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "method")
                .unwrap()
                .value,
            "submit_social_link"
        );
    }

    #[test]
    fn query_by_social() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                accepted_token: vec!["ucmst".to_string()],
            },
        )
        .unwrap();
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetAddressesBySocial {
                profile_id: "1".to_string(),
                platform: "twitter".to_string(),
            },
        );
        assert!(resp.is_ok());
        let resp: GetAddressesBySocialResponse = from_binary(&resp.unwrap()).unwrap();

        assert_eq!(resp, GetAddressesBySocialResponse { address: vec![] });
        // execute
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            ExecuteMsg::SubmitSocial {
                social_info: ("twitter".to_string(), "123".to_string()),
                address: Addr::unchecked("abc"),
            },
        );
        assert!(resp.is_ok());
        // query again
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetAddressesBySocial {
                platform: "twitter".to_string(),
                profile_id: "123".to_string(),
            },
        )
        .unwrap();
        let resp: GetAddressesBySocialResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            GetAddressesBySocialResponse {
                address: vec![Addr::unchecked("abc")]
            }
        );
        // execute
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            ExecuteMsg::SubmitSocial {
                social_info: ("facebook".to_string(), "456".to_string()),
                address: Addr::unchecked("abc"),
            },
        );
        assert!(resp.is_ok());
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetAddressesBySocial {
                platform: "facebook".to_string(),
                profile_id: "456".to_string(),
            },
        )
        .unwrap();
        let resp: GetAddressesBySocialResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            GetAddressesBySocialResponse {
                address: vec![Addr::unchecked("abc")]
            }
        );
    }
    #[test]
    fn query_by_address() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {
                accepted_token: vec!["ucmst".to_string()],
            },
        )
        .unwrap();
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetSocialsByAddress {
                address: Addr::unchecked("owner"),
            },
        );
        assert!(resp.is_ok());
        let resp: GetSocialsByAddressResponse = from_binary(&resp.unwrap()).unwrap();

        assert_eq!(
            resp,
            GetSocialsByAddressResponse {
                social_infos: vec![]
            }
        );
        // execute
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            ExecuteMsg::SubmitSocial {
                social_info: ("twitter".to_string(), "123".to_string()),
                address: Addr::unchecked("user"),
            },
        );
        assert!(resp.is_ok());
        // query again
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetSocialsByAddress {
                address: Addr::unchecked("user"),
            },
        )
        .unwrap();
        let resp: GetSocialsByAddressResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            GetSocialsByAddressResponse {
                social_infos: vec![("twitter".to_string(), "123".to_string())]
            }
        );
        // execute
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            ExecuteMsg::SubmitSocial {
                social_info: ("facebook".to_string(), "456".to_string()),
                address: Addr::unchecked("user"),
            },
        );
        assert!(resp.is_ok());
        // query again
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetSocialsByAddress {
                address: Addr::unchecked("user"),
            },
        )
        .unwrap();
        let resp: GetSocialsByAddressResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            GetSocialsByAddressResponse {
                social_infos: vec![
                    ("facebook".to_string(), "456".to_string()),
                    ("twitter".to_string(), "123".to_string()),
                ]
            }
        );
    }
    #[test]
    fn donations() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(100, "ucmst"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    accepted_token: vec!["ucmst".to_string(), "eth".to_string()],
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {
                recipient: Addr::unchecked("admin1"),
            },
            &coins(100, "ucmst"),
        )
        .unwrap();
        assert_eq!(
            app.wrap()
                .query_balance("user", "ucmst")
                .unwrap()
                .amount
                .u128(),
            0
        );
        // Verify that the fees and recipient amounts have been sent accurately.
        assert_eq!(
            app.wrap()
                .query_balance("admin1", "ucmst")
                .unwrap()
                .amount
                .u128(),
            95
        );
        assert_eq!(
            app.wrap()
                .query_balance("owner", "ucmst")
                .unwrap()
                .amount
                .u128(),
            5
        );
    }
}
