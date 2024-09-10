use crate::{
    error::ContractError,
    math::sub_u128,
    msg::{CurveAmmOverview, CurveFeeOverview, CurveStatsOverview, OverviewResponse},
    state::{
        BASE_TOKEN, CURVE, FEE_ADDR, FEE_PCT_BUY, FEE_PCT_SELL, MAKER_STATS, NET_MAKER_FEE, NET_TAKER_FEE,
        QUOTE_RESERVE_VIRTUAL, QUOTE_TOKEN, TAKER_STATS,
    },
};

use super::ReadonlyContext;

pub fn query_overview(ctx: ReadonlyContext) -> Result<OverviewResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let store = deps.storage;

    let curve = CURVE.load(store)?;
    let vl = QUOTE_RESERVE_VIRTUAL.load(store)?;

    Ok(OverviewResponse {
        amm: CurveAmmOverview {
            base_token: BASE_TOKEN.load(store)?,
            base_reserve: curve.base_reserve,
            base_decimals: curve.base_decimals,
            quote_token: QUOTE_TOKEN.load(store)?,
            quote_reserve_real: sub_u128(curve.quote_reserve, vl)?,
            quote_reserve_virtual: vl,
            quote_decimals: curve.quote_decimals,
            constant_product: curve.k,
        },
        fees: CurveFeeOverview {
            recipient: FEE_ADDR.load(store)?,
            taker_pct: FEE_PCT_BUY.load(store)?,
            maker_pct: FEE_PCT_SELL.load(store)?,
        },
        stats: CurveStatsOverview {
            bids: TAKER_STATS.load(store)?,
            asks: MAKER_STATS.load(store)?,
            net_maker_fee: NET_MAKER_FEE.load(store)?,
            net_taker_fee: NET_TAKER_FEE.load(store)?,
        },
    })
}
