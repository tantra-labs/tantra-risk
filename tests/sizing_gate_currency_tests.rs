//! The sizing gate refuses a budget and account equity in different
//! currencies instead of comparing their bare amounts (root gap 0014).

use rust_decimal::Decimal;
use tantra_core::{
    Currency, Exchange, InstrumentId, Money, PositionSide, PositionSizingRequest, Price, Quantity,
    RiskBudget, SizingMethod,
};
use tantra_risk::sizing_gate::SizingRiskRefusal;
use tantra_risk::{RiskError, validate_sizing_request};

fn request_with(
    account_equity: Money,
    budget_equity: Money,
) -> (PositionSizingRequest, RiskBudget) {
    let instrument = InstrumentId::try_new(Exchange::Nse, "RELIANCE").unwrap();
    let risk_pct = Decimal::from_str_radix("0.01", 10).unwrap();
    let budget = RiskBudget::new(budget_equity, risk_pct, Quantity::from_u64(1));
    let request = PositionSizingRequest::new(
        instrument,
        PositionSide::Long,
        Price::try_new(Decimal::from(2500)).unwrap(),
        account_equity,
        budget.clone(),
        SizingMethod::FixedFractional { risk_pct },
    )
    .with_stop_loss(Price::try_new(Decimal::from(2450)).unwrap());
    (request, budget)
}

#[test]
fn mismatched_budget_and_account_currency_is_refused() {
    let (request, budget) = request_with(
        Money::new(Decimal::from(1_000_000), Currency::Usd),
        Money::inr(Decimal::from(1_000_000)),
    );
    match validate_sizing_request(&request, &budget) {
        Err(RiskError::SizingRefusal(SizingRiskRefusal::InvalidEquity { reason, .. })) => {
            assert!(reason.contains("currency mismatch"), "reason: {reason}");
        }
        other => panic!("expected InvalidEquity currency refusal, got {other:?}"),
    }
}

#[test]
fn same_currency_budget_and_account_passes_the_equity_check() {
    let equity = Money::inr(Decimal::from(1_000_000));
    let (request, budget) = request_with(equity, equity);
    assert!(
        !matches!(
            validate_sizing_request(&request, &budget),
            Err(RiskError::SizingRefusal(
                SizingRiskRefusal::InvalidEquity { .. }
            ))
        ),
        "same-currency equity must not be refused as invalid"
    );
}
