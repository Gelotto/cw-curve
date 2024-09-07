use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Uint256};

use crate::{
    error::ContractError,
    math::{add_u128, div_u256, mul_ratio_u128, sub_u128},
};

#[cw_serde]
pub struct Curve {
    pub k: Uint256,
    pub base_reserve: Uint128,
    pub base_decimals: u8,
    pub quote_reserve: Uint128,
    pub quote_decimals: u8,
}

impl Curve {
    pub fn buy(
        &mut self,
        in_amount: Uint128,
        min_amount_out: Option<Uint128>,
    ) -> Result<Uint128, ContractError> {
        let (new_quote_reserve, new_base_reserve, out_amount) = {
            let new_quote_reserve = add_u128(self.quote_reserve, in_amount)?;
            let new_base_reserve = div_u256(self.k, new_quote_reserve)?.try_into().unwrap();
            let out_amount = sub_u128(self.base_reserve, new_base_reserve)?;
            (new_quote_reserve, new_base_reserve, out_amount)
        };

        self.base_reserve = new_base_reserve;
        self.quote_reserve = new_quote_reserve;

        // Enforce slippage protection
        if let Some(min_amount_out) = min_amount_out {
            if out_amount < min_amount_out {
                return Err(ContractError::TooMuchSlippage {});
            }
        }

        Ok(out_amount)
    }

    pub fn sell(
        &mut self,
        in_amount: Uint128,
        min_amount_out: Option<Uint128>,
    ) -> Result<Uint128, ContractError> {
        let (new_quote_reserve, new_base_reserve, out_amount) = {
            let new_base_reserve = add_u128(self.base_reserve, in_amount)?;
            let new_quote_reserve = div_u256(self.k, new_base_reserve)?.try_into().unwrap();
            let out_amount = sub_u128(self.quote_reserve, new_quote_reserve)?;
            (new_quote_reserve, new_base_reserve, out_amount)
        };

        self.base_reserve = new_base_reserve;
        self.quote_reserve = new_quote_reserve;

        // Enforce slippage protection
        if let Some(min_amount_out) = min_amount_out {
            if out_amount < min_amount_out {
                return Err(ContractError::TooMuchSlippage {});
            }
        }

        Ok(out_amount)
    }

    /// Calculates BASE price with respect to QUOTE
    pub fn calculate_quote_price(&self) -> Result<Uint128, ContractError> {
        mul_ratio_u128(
            self.quote_reserve,
            10u128.pow(self.quote_decimals as u32),
            self.base_reserve,
        )
    }

    pub fn to_base_amount(
        &self,
        quote_amount: Uint128,
    ) -> Result<Uint128, ContractError> {
        mul_ratio_u128(
            quote_amount,
            10u128.pow(self.quote_decimals as u32),
            self.calculate_base_price()?,
        )
    }

    pub fn to_quote_amount(
        &self,
        base_amount: Uint128,
    ) -> Result<Uint128, ContractError> {
        mul_ratio_u128(
            base_amount,
            10u128.pow(self.base_decimals as u32),
            self.calculate_quote_price()?,
        )
    }

    /// Calculates QUOTE price with respect to BASE
    pub fn calculate_base_price(&self) -> Result<Uint128, ContractError> {
        mul_ratio_u128(
            self.base_reserve,
            10u128.pow(self.quote_decimals as u32),
            self.quote_reserve,
        )
    }
}
