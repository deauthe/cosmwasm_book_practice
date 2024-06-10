use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use error::ContractError;
use msg::InstantiateMsg;

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    msg: msg::ExecuteMsg,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    contract::execute(deps, env, info, msg)
}

//The cfg_attr attribute is a conditional compilation attribute, similar to the cfg we used before for the test.
//  It expands to the given attribute if the condition expands to true. In our case - 
// it would expand to nothing if the feature "library" is enabled, or it would expand just to #[entry_point] in another case.