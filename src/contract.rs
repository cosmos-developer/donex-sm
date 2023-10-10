#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SocialResponse};
use crate::state::{Platform, ProfileId, SocialInfo, ADDRESS_TO_SOCIAL, OWNER, SOCIAL_TO_ADDRESS};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "cosmos:donex-sm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(_deps.storage, &info.sender)?;
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
    match msg {
        ExecuteMsg::SubmitSocial {
            social_info,
            address,
        } => submit_social_link(deps, info, social_info, address),
        // ExecuteMsg::Reset { count } => try_reset(deps, info, count),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSocial {
            profile_id,
            platform,
        } => to_binary(&query_by_social_link(deps, profile_id, platform)?),
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
    let social_to_address = SOCIAL_TO_ADDRESS.load(deps.storage, social_info.clone());
    let address_to_social = ADDRESS_TO_SOCIAL.load(deps.storage, info.sender.clone());

    if social_to_address.is_ok() {
        return Err(ContractError::SocialAlreadyLinked {});
    }
    if address_to_social.is_ok() {
        return Err(ContractError::AddressAlreadyLinked {});
    }
    SOCIAL_TO_ADDRESS.save(deps.storage, social_info.clone(), &address)?;
    ADDRESS_TO_SOCIAL.save(deps.storage, address, &social_info)?;
    Ok(Response::new().add_attribute("method", "submit_social_link"))
}
fn query_by_social_link(
    deps: Deps,
    profile_id: ProfileId,
    platform: Platform,
) -> StdResult<SocialResponse> {
    let social_info = (platform, profile_id);
    let address = SOCIAL_TO_ADDRESS.load(deps.storage, social_info.clone())?;
    Ok(SocialResponse { address })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr, Empty};
    // #[test]
    // fn submit_social() {
    //     let mut app = App::default();

    //     let code = ContractWrapper::new(execute, instantiate, query);
    //     let code_id = app.store_code(Box::new(code));

    //     let addr = app
    //         .instantiate_contract(
    //             code_id,
    //             Addr::unchecked("owner"),
    //             &Empty {},
    //             &[],
    //             "Contract",
    //             None,
    //         )
    //         .unwrap();
    //     let resp = app
    //         .execute_contract(
    //             addr.clone(),
    //             Addr::unchecked("owner"),
    //             &ExecuteMsg::SubmitSocial {
    //                 social_info: ("twitter".to_string(), "123".to_string()),
    //                 address: Addr::unchecked("abcde"),
    //             },
    //             &[],
    //         )
    //         .unwrap();
    //     let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    //     assert_eq!(
    //         wasm.attributes
    //             .iter()
    //             .find(|attr| attr.key == "action")
    //             .unwrap()
    //             .value,
    //         "submit_social_link"
    //     );
    // }

    #[test]
    fn social_query() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InstantiateMsg {},
        )
        .unwrap();
        let resp = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetSocial {
                profile_id: "1".to_string(),
                platform: "twitter".to_string(),
            },
        );
        assert!(resp.is_err());
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
            QueryMsg::GetSocial {
                profile_id: "123".to_string(),
                platform: "twitter".to_string(),
            },
        )
        .unwrap();
        let resp: SocialResponse = from_binary(&resp).unwrap();
        assert_eq!(
            resp,
            SocialResponse {
                address: Addr::unchecked("abc")
            }
        );
    }
}