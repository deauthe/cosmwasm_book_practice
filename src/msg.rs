use cosmwasm_schema::{cw_serde,QueryResponses};
use cosmwasm_std::Addr;


//the #[cw_serde] macro does the following:
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
// #[derive(Serialize,Deserialize,PartialEq,Debug,Clone,JsonSchema)]




#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GreetResp)]
    Greet {},
    #[returns(AdminsListResp)]
    AdminsList {},
}
// QueryResponses trait for our query message to correlate
//  the message variants with responses we would generate for them
//this just means that Greet (Unit like variant of an enum) will return A Greet Response when supplied in a query,
//the return macro tells so and it is there to build a good documentation and know the expected type to later generate a schema 
//out of the expected types 


#[cw_serde]
pub struct GreetResp {
    pub message: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub donation_denom: String,
}

#[cw_serde]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMembers { admins: Vec<String> },
    Leave {},
    Donate {},
}
