use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Account {
    pub stats: AccountStats,
}

#[cw_serde]
pub struct AccountStats {
    pub n_buys: u32,
    pub n_sells: u32,
}
