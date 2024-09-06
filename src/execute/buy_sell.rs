use cosmwasm_std::{attr, Addr, Response, Uint128};

use crate::{
    error::ContractError,
    math::{add_u32, mul_pct_u128, sub_u128},
    models::{account::AccountStats, ohlc::OhlcBar},
    state::{ACCOUNT_STATS, BUY_FEE_PCT, CURVE, FEE_RECIPIENT_ADDR, QUOTE_TOKEN, SELL_FEE_PCT},
    utils::resolve_initiator,
};

use super::Context;

pub fn exec_buy(
    ctx: Context,
    initiator: Option<Addr>,
    amount: Uint128,
    is_buy: bool,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let quote_token = QUOTE_TOKEN.load(deps.storage)?;
    let mut curve = CURVE.load(deps.storage)?;

    // Compute buy or sell-side platform fee
    let fee_amount = mul_pct_u128(
        amount,
        if is_buy {
            BUY_FEE_PCT.load(deps.storage)?
        } else {
            SELL_FEE_PCT.load(deps.storage)?
        },
    )?;

    // Subtract fee from amount recieved by sender
    let in_amount = sub_u128(amount, fee_amount)?;

    // Perform AMM swap
    let out_amount = if is_buy {
        curve.buy(in_amount)
    } else {
        curve.sell(in_amount)
    }?;

    // Update initiator's account info
    let initiator = resolve_initiator(deps.storage, deps.api, &info.sender, initiator)?;
    ACCOUNT_STATS.update(deps.storage, &initiator, |maybe_stats| -> Result<_, ContractError> {
        let mut stats = maybe_stats.unwrap_or_else(|| AccountStats { n_buys: 0, n_sells: 0 });
        if is_buy {
            stats.n_buys = add_u32(stats.n_buys, 1)?;
        } else {
            stats.n_sells = add_u32(stats.n_sells, 1)?;
        }
        Ok(stats)
    })?;

    // Update candlestick data
    OhlcBar::upsert(
        deps.storage,
        env.block.time,
        curve.calculate_spot_price()?,
        out_amount,
        in_amount,
    )?;

    let mut resp = Response::new().add_attributes(vec![
        attr("action", "buy"),
        attr("in_amount", amount.to_string()),
        attr("out_amount", out_amount.to_string()),
    ]);

    // Add submsg to send platform fee if exists
    if !fee_amount.is_zero() {
        let fee_recipient = FEE_RECIPIENT_ADDR.load(deps.storage)?;
        resp = resp.add_submessage(quote_token.transfer(&fee_recipient, fee_amount)?);
    }

    Ok(resp)
}
