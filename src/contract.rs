use crate::error::ContractError;
use crate::msg::{AdminsListResp, ExecuteMsg, GreetResp, InstantiateMsg, QueryMsg};
use crate::state::{ADMINS, DONATION_DENOM};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_utils;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins: StdResult<Vec<_>> = msg
        .admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect();
    ADMINS.save(deps.storage, &admins?)?;
    //deps.storage is actual blockchain storage, it's type is &mut Storage; The second argument is the data to be stored.
    //ADMINS is just an accesso for the storage and not the actual storage. This helps us to manage and manipulate the storage.

    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Greet {} => to_json_binary(&query::greet()?),
        AdminsList {} => to_json_binary(&query::admins_list(deps)?),
    }
}

#[allow(dead_code)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*; //bringing inside into scope

    match msg {
        AddMembers { admins } => exec::add_members(deps, info, admins),
        Leave {} => exec::leave(deps, info).map_err(Into::into),
        Donate {} => exec::donate(deps, info),
    }
}

mod exec {
    use super::*;
    use cosmwasm_std::{coins, BankMsg, Event, StdError, Uint128};

    pub fn add_members(
        deps: DepsMut,
        info: MessageInfo,
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let mut curr_admins = ADMINS.load(deps.storage)?;
        if !curr_admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

        let events = admins
            .iter()
            .map(|admin| Event::new("admin_added").add_attribute("addr", admin));
        //every contract emits events, we can also add custom ones like the one above
        //every event has an event type from the "new" function and an attribute.
        //attributes are key value pairs
        //Because an event cannot contain any list, to achieve reporting multiple similar actions taking place,
        //we need to emit multiple small events instead of a collective one.
        //every execution by default emits a std wasm event. Adding Attributes to the
        //result just adds them to the default event

        let resp = Response::new()
            .add_events(events)
            .add_attribute("action", "add_members")
            .add_attribute("added_count", admins.len().to_string());

        let admins: StdResult<Vec<_>> = admins
            .into_iter()
            .map(|addr| deps.api.addr_validate(&addr))
            .collect();
        curr_admins.append(&mut admins?);
        ADMINS.save(deps.storage, &curr_admins)?;

        Ok(resp)
    }
    pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        ADMINS.update(deps.storage, move |admins| -> StdResult<_> {
            let admins = admins
                .into_iter()
                .filter(|admin| *admin != info.sender)
                .collect();
            Ok(admins)
        })?;

        Ok(Response::new())
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let denom = DONATION_DENOM.load(deps.storage)?;
        let admins = ADMINS.load(deps.storage)?;

        let donation = cw_utils::must_pay(&info, &denom)?.u128();

        let donation_per_admin = donation / (admins.len() as u128);

        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.to_string(),
            amount: coins(donation_per_admin, &denom),
        });

        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("action", "donate")
            .add_attribute("amount", donation.to_string())
            .add_attribute("per_admin", donation_per_admin.to_string());

        Ok(resp)
    }
}

mod query {
    use cosmwasm_std::Addr;

    use super::*;

    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello World".to_owned(),
        };
        Ok(resp)
    }
    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let admins: Vec<Addr> = ADMINS.load(deps.storage)?;
        let resp = AdminsListResp { admins };
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::msg::AdminsListResp;

    use super::*;

    #[test]
    fn greet_query() {
        let mut app: App = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code)); //simulate the storing code on the blockchain

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec![],
                    donation_denom: "donation_denom".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: GreetResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Greet {})
            .unwrap();
        //to query a smart contract, it's required to use the wrap method as it's an accessor for these. We need to know
        //that query calls are different than other methods and wraps helps us to make these api.
        //we're just querying the greet response from the contract here

        assert_eq!(
            resp,
            GreetResp {
                message: "Hello World".to_owned(),
            }
        );
    }

    #[test]
    fn instantantiation() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        //now instantiating 2 different contracts with different messages and querying them to see if
        //they show the desired behaviour

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec![],
                    donation_denom: "donation_denom".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp, AdminsListResp { admins: vec![] });

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    donation_denom: "donation_denom".to_owned(),
                },
                &[],
                "Contract2",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(
            resp,
            AdminsListResp {
                admins: vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")]
            }
        )
    }

    #[test]
    fn unauthorized() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    admins: vec![],
                    donation_denom: "donation_denom".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("user"),
                addr,
                &ExecuteMsg::AddMembers {
                    admins: vec!["user".to_owned()],
                },
                &[],
            )
            .unwrap_err();
        //the error type returned from this is anyhow::Error as the execute function
        //is from the cosmwasm library and not our own.
        //anyhow errors can recover their original type using the downcast function. The unwrap right after it is needed because downcasting may fail.
        // The reason is that downcast doesn't magically know the type kept in the underlying error. It deduces it by some context -
        // here, it knows we expect it to be a ContractError, because of being compared to it - type elision miracles. But if the underlying error would not be a ContractError,
        // then unwrap would panic.

        assert_eq!(
            ContractError::Unauthorized {
                sender: Addr::unchecked("user")
            },
            err.downcast().unwrap()
        )
    }

    #[test]
    fn add_members() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec!["owner".to_owned()],
                    donation_denom: "donation_denom".to_owned(),
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
                &ExecuteMsg::AddMembers {
                    admins: vec!["user".to_owned()],
                },
                &[],
            )
            .unwrap();

        let wasm = resp.events.iter().find(|event| event.ty == "wasm").unwrap();
        //ty stands for type, shitty but it is whatit is ;
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "add_members"
        );

        assert_eq!(
            (wasm
                .attributes
                .iter()
                .find(|attr| attr.key == "added_count")
                .unwrap()
                .value),
            "1"
        );

        let admin_added: Vec<_> = resp
            .events
            .iter()
            .filter(|event| event.ty == "wasm-admin_added")
            .collect();
        assert_eq!(
            admin_added[0]
                .attributes
                .iter()
                .find(|attr| attr.key == "addr")
                .unwrap()
                .value,
            "user"
        );

        // the "wasm" event is particularly tricky, as it contains an implied attribute -
        //_contract_addr containing an address called a contract. My general rule is -
        // do not test emitted events unless some logic depends on them.

        //Besides events, any smart contract execution may produce a data object.
        // In contrast to events, data can be structured.
        //It makes it a way better choice to perform any communication logic relies on.
        // On the other hand, it turns out it is very rarely helpful outside of contract-to-contract communication.
    }

    #[test]
    fn donations() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(5, "eth"))
                .unwrap()
        });
        //app creation is different in this fn because we need an initial balance in the function
        // Unfortunately, new documentation is not easy to follow - even if a function is not very complicated.
        //What it takes as an argument is a closure with three arguments - the Router with all modules supported by multi-test,
        //the API object, and the state
        // The router object contains some generic fields - we are interested in bank in particular. It has a type of BankKeeper,
        //where the init_balance function sits.

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec!["admins".to_owned(), "admin2".to_owned()],
                    donation_denom: "eth".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "eth"),
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            1
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin1", "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin2", "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );
    }
}

//The contract we built has an exploitable bug. All donations are distributed equally across admins. However,
// every admin is eligible to add another admin.
// And nothing is preventing the admin from adding himself to the list and receiving twice as many rewards as others!