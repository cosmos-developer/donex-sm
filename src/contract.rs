#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetAddressesBySocialResponse, GetSocialsByAddressResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{Platform, ProfileId, SocialInfo, UserInfo, ACCEPTED_TOKEN, OWNER, USER_INFOS};
const CONTRACT_NAME: &str = "cosmos:donex-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        SubmitSocial {
            social_info,
            address,
        } => submit_social_link(deps, info, social_info, address),
        Donate { .. } => todo!(),
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
    let user_info = UserInfo {
        address: address.clone(),
        platform_id: social_info.0,
        profile_id: social_info.1,
    };
    USER_INFOS.save(deps.storage, address.as_ref(), &user_info)?;

    Ok(Response::new().add_attribute("method", "submit_social_link"))
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
        .prefix(address)
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
                    accepted_token: vec!["ATOM".to_string()],
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
                accepted_token: vec!["ATOM".to_string()],
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
            env,
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
                accepted_token: vec!["ATOM".to_string()],
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
    }
}
