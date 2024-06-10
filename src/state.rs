use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");
pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");

// Every one of such constants represents a single portion of the contract state - as tables in databases. 
// The types of those constants represent what kind of table this is. The most basic ones are Item<T>,
//  which keeps zero or one element of a given type, and Map<K, T> which is a key-value map.