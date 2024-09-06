use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Storage, Timestamp, Uint128, Uint64};

use crate::{
    error::ContractError,
    math::{add_u128, add_u32},
    state::OHLC_BARS,
};

#[cw_serde]
pub struct OhlcBar {
    pub o: Uint128,
    pub c: Uint128,
    pub h: Uint128,
    pub l: Uint128,
    pub vb: Uint128,
    pub vq: Uint128,
    pub t: Uint64,
    pub n: u32,
}

impl OhlcBar {
    pub fn new(t: Uint64) -> Self {
        Self {
            o: Uint128::zero(),
            h: Uint128::zero(),
            l: Uint128::zero(),
            c: Uint128::zero(),
            vb: Uint128::zero(),
            vq: Uint128::zero(),
            n: 0,
            t,
        }
    }

    pub fn upsert(
        store: &mut dyn Storage,
        time: Timestamp,
        price: Uint128,
        base_volume: Uint128,
        quote_volume: Uint128,
    ) -> Result<OhlcBar, ContractError> {
        let seconds = time.seconds();
        let t = seconds - (seconds % 60);
        OHLC_BARS.update(store, t, |maybe_bar| -> Result<_, ContractError> {
            let mut bar = maybe_bar.unwrap_or_else(|| OhlcBar::new(t.into()));
            if bar.n > 0 {
                if price > bar.h {
                    bar.h = price;
                }
                if price < bar.l {
                    bar.l = price;
                }
            } else {
                bar.o = price;
                bar.h = price;
                bar.l = price;
            }
            bar.c = price;
            bar.vq = add_u128(bar.vq, quote_volume)?;
            bar.vb = add_u128(bar.vb, base_volume)?;
            bar.n = add_u32(bar.n, 1)?;
            Ok(bar)
        })
    }
}
