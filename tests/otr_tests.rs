//! OtrTracker sliding-window unit tests.
//!
//! Extracted from `src/otr.rs` during Tier 1A migration per the
//! Parallel Test Directory Rule: zero `#[cfg(test)]` modules inside `src/`.

use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use tantra_risk::{OtrConfig, OtrTracker};

#[test]
fn test_cumulative_otr() {
    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Place 10 orders, 1 fill -> OTR = 10
    for _ in 0..10 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);

    assert_eq!(tracker.total_order_operations(), 10);
    assert_eq!(tracker.current_otr(), Decimal::from(10));
}

#[test]
fn test_sliding_window_otr() {
    let mut tracker = OtrTracker::new();
    let base = Utc.with_ymd_and_hms(2025, 8, 1, 10, 0, 0).unwrap();

    // Place 5 orders at t=0
    for _ in 0..5 {
        tracker.record_order_placement(base);
    }
    // 1 fill at t=0
    tracker.record_fill(base);

    // 5 more orders at t=55 seconds (within 1-minute window)
    let t55 = base + chrono::Duration::seconds(55);
    for _ in 0..5 {
        tracker.record_order_placement(t55);
    }

    // At t=55s, 1-min window should show 10 ops, 1 trade = OTR 10
    let otr = tracker.window_otr_1min(t55, &OtrConfig::default());
    assert_eq!(otr, Decimal::from(10));

    // At t=61 seconds, the t=0 events fall out of the 1-minute window
    let t61 = base + chrono::Duration::seconds(61);
    let otr = tracker.window_otr_1min(t61, &OtrConfig::default());
    // Only the 5 orders at t=55s remain in window, no trades = OTR 5
    assert_eq!(otr, Decimal::from(5));
}

#[test]
fn test_is_breached_grace_period() {
    let config = OtrConfig {
        max_otr: Decimal::from(10),
        min_orders_before_enforcement: 5,
        ..Default::default()
    };

    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Place 3 orders, 0 fills (below grace period)
    for _ in 0..3 {
        tracker.record_order_placement(now);
    }
    assert!(!tracker.is_breached(&config));

    // Place 2 more (total 5, meets grace period), OTR = 5/1 = 5 (below max 10)
    for _ in 0..2 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);
    assert!(!tracker.is_breached(&config));
}

#[test]
fn test_is_breached_exceeded() {
    let config = OtrConfig {
        max_otr: Decimal::from(10),
        min_orders_before_enforcement: 5,
        ..Default::default()
    };

    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Place 11 orders, 1 fill -> OTR = 11 > max 10
    for _ in 0..11 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);
    assert!(tracker.is_breached(&config));
}

#[test]
fn test_warning_ratio() {
    let config = OtrConfig {
        max_otr: Decimal::from(50),
        warning_threshold: Decimal::from(40),
        ..Default::default()
    };

    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Place 45 orders, 1 fill -> OTR = 45, in warning zone (>40, <=50)
    for _ in 0..45 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);

    assert_eq!(tracker.warning_ratio(&config), Some(Decimal::from(45)));

    // Place 55 orders, 1 fill -> OTR = 55, above max, no warning
    let mut tracker2 = OtrTracker::new();
    for _ in 0..55 {
        tracker2.record_order_placement(now);
    }
    tracker2.record_fill(now);
    assert_eq!(tracker2.warning_ratio(&config), None);
}

#[test]
fn test_total_order_operations_with_all_three_types() {
    let mut tracker = OtrTracker::new();
    let now = Utc::now();
    // 5 placed, 4 modified, 3 cancelled
    for _ in 0..5 {
        tracker.record_order_placement(now);
    }
    tracker.orders_modified = 4;
    for _ in 0..3 {
        tracker.record_order_mod_or_cancel(now);
    }
    // Total must be 5 + 4 + 3 = 12. If (5 - 4) + 3, it would be 4!
    assert_eq!(tracker.total_order_operations(), 12);
}

#[test]
fn test_window_otr_exact_cutoff_and_multiple_trades() {
    let mut tracker = OtrTracker::new();
    let now = Utc::now();
    let window_ms = 60_000u64;
    let cutoff = now - chrono::Duration::milliseconds(window_ms as i64);

    // Place an event at EXACTLY cutoff
    tracker.record_order_placement(cutoff);
    // And place 5 orders and 2 fills inside the window
    for _ in 0..5 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);
    tracker.record_fill(now);

    // Events at cutoff must NOT be evicted: total ops = 1 (at cutoff) + 5 = 6 ops.
    // Trades in window = 2.
    // OTR = 6 / 2 = 3.
    // If / was *, it would be 6 * 2 = 12!
    // If < was <=, cutoff event would be evicted (5 ops / 2 = 2.5)!
    let otr = tracker.window_otr(now, window_ms);
    assert_eq!(otr, Decimal::from(3));
}

#[test]
fn test_is_breached_grace_period_high_ratio() {
    let config = OtrConfig {
        max_otr: Decimal::from(2),
        min_orders_before_enforcement: 5,
        ..Default::default()
    };
    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // 3 orders, 0 fills -> current OTR is 3 > max_otr (2)
    // But 3 < 5 (min_orders_before_enforcement), so it MUST NOT be breached.
    for _ in 0..3 {
        tracker.record_order_placement(now);
    }
    assert_eq!(tracker.current_otr(), Decimal::from(3));
    assert!(!tracker.is_breached(&config));

    // 4 orders, 0 fills -> 4 < 5, still NOT breached
    tracker.record_order_placement(now);
    assert_eq!(tracker.total_order_operations(), 4);
    assert!(!tracker.is_breached(&config));

    // 5 orders, 0 fills -> 5 >= 5 and OTR 5 > 2 -> NOW breached
    tracker.record_order_placement(now);
    assert_eq!(tracker.total_order_operations(), 5);
    assert!(tracker.is_breached(&config));
}

#[test]
fn test_record_cancellations_and_modifications() {
    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // 3 placements, 2 cancellations
    tracker.record_order_placement(now);
    tracker.record_order_placement(now);
    tracker.record_order_placement(now);
    tracker.record_order_mod_or_cancel(now);
    tracker.record_order_mod_or_cancel(now);

    assert_eq!(tracker.orders_placed, 3);
    assert_eq!(tracker.orders_cancelled, 2);
    assert_eq!(tracker.total_order_operations(), 5);
    assert_eq!(tracker.event_count(), 5);
    // 5 ops, 0 trades -> OTR = 5 / 1 = 5
    assert_eq!(tracker.current_otr(), Decimal::from(5));
}

#[test]
fn test_multiple_fills_and_division() {
    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    for _ in 0..12 {
        tracker.record_order_placement(now);
    }
    // 0 fills -> 12 / 1 = 12
    assert_eq!(tracker.current_otr(), Decimal::from(12));

    tracker.record_fill(now);
    // 1 fill -> 12 / 1 = 12
    assert_eq!(tracker.current_otr(), Decimal::from(12));

    tracker.record_fill(now);
    // 2 fills -> 12 / 2 = 6
    assert_eq!(tracker.current_otr(), Decimal::from(6));

    tracker.record_fill(now);
    // 3 fills -> 12 / 3 = 4
    assert_eq!(tracker.current_otr(), Decimal::from(4));
    assert_eq!(tracker.event_count(), 15);
}

#[test]
fn test_window_otr_5min_and_boundary_eviction() {
    let mut tracker = OtrTracker::new();
    let base = Utc.with_ymd_and_hms(2025, 8, 1, 10, 0, 0).unwrap();

    // Event at base (t=0)
    tracker.record_order_placement(base);

    // Event at 3 minutes (t=180s)
    let t3m = base + chrono::Duration::seconds(180);
    tracker.record_order_placement(t3m);
    tracker.record_fill(t3m);

    // Check at t=200s (both events in 5m window)
    let t200s = base + chrono::Duration::seconds(200);
    // 2 ops, 1 fill -> OTR 2
    assert_eq!(
        tracker.window_otr_5min(t200s, &OtrConfig::default()),
        Decimal::from(2)
    );

    // Check at t=301s (5 minutes and 1 second later):
    // t=0 event evicted (> 300,000ms), only t=180s event remains: 1 op, 1 trade -> OTR 1
    let t301s = base + chrono::Duration::seconds(301);
    assert_eq!(
        tracker.window_otr_5min(t301s, &OtrConfig::default()),
        Decimal::from(1)
    );
}

#[test]
fn test_is_breached_boundary_conditions() {
    let config = OtrConfig {
        max_otr: Decimal::from(10),
        min_orders_before_enforcement: 5,
        ..Default::default()
    };

    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Exactly 4 orders (1 below enforcement) with OTR 4/1 = 4
    for _ in 0..4 {
        tracker.record_order_placement(now);
    }
    assert!(!tracker.is_breached(&config));

    // Exactly 5 orders (meets enforcement), OTR = 5 <= 10 -> not breached
    tracker.record_order_placement(now);
    assert_eq!(tracker.total_order_operations(), 5);
    assert!(!tracker.is_breached(&config));

    // 10 orders, 1 fill -> OTR = 10 == max_otr -> NOT breached (strictly >)
    for _ in 0..5 {
        tracker.record_order_placement(now);
    }
    tracker.record_fill(now);
    assert_eq!(tracker.current_otr(), Decimal::from(10));
    assert!(!tracker.is_breached(&config));

    // 11 orders, 1 fill -> OTR = 11 > max_otr -> BREACHED!
    tracker.record_order_placement(now);
    assert_eq!(tracker.current_otr(), Decimal::from(11));
    assert!(tracker.is_breached(&config));
}

#[test]
fn test_warning_ratio_exact_boundaries() {
    let config = OtrConfig {
        max_otr: Decimal::from(50),
        warning_threshold: Decimal::from(40),
        ..Default::default()
    };

    let mut tracker = OtrTracker::new();
    let now = Utc::now();

    // Exactly 40 orders -> OTR 40 == warning_threshold -> None (must be strictly > 40)
    for _ in 0..40 {
        tracker.record_order_placement(now);
    }
    assert_eq!(tracker.current_otr(), Decimal::from(40));
    assert_eq!(tracker.warning_ratio(&config), None);

    // 41 orders -> OTR 41 > 40 and <= 50 -> Some(41)
    tracker.record_order_placement(now);
    assert_eq!(tracker.current_otr(), Decimal::from(41));
    assert_eq!(tracker.warning_ratio(&config), Some(Decimal::from(41)));

    // 50 orders -> OTR 50 == max_otr -> Some(50)
    for _ in 0..9 {
        tracker.record_order_placement(now);
    }
    assert_eq!(tracker.current_otr(), Decimal::from(50));
    assert_eq!(tracker.warning_ratio(&config), Some(Decimal::from(50)));

    // 51 orders -> OTR 51 > max_otr -> None (breached, not warning)
    tracker.record_order_placement(now);
    assert_eq!(tracker.current_otr(), Decimal::from(51));
    assert_eq!(tracker.warning_ratio(&config), None);
}
