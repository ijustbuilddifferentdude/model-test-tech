use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Snapshot {
    pub local_timestamp: f64,

    // best 5 bids
    pub bid_price_1: f64,
    pub bid_price_2: f64,
    pub bid_price_3: f64,
    pub bid_price_4: f64,
    pub bid_price_5: f64,
    pub bid_qty_1: f64,
    pub bid_qty_2: f64,
    pub bid_qty_3: f64,
    pub bid_qty_4: f64,
    pub bid_qty_5: f64,

    // best 5 asks
    pub ask_price_1: f64,
    pub ask_price_2: f64,
    pub ask_price_3: f64,
    pub ask_price_4: f64,
    pub ask_price_5: f64,
    pub ask_qty_1: f64,
    pub ask_qty_2: f64,
    pub ask_qty_3: f64,
    pub ask_qty_4: f64,
    pub ask_qty_5: f64,
}

#[derive(Debug, Clone)]
pub struct Feat {
    pub ts_ms: f64,
    pub mid: f64,
    pub spread: f64,
    pub rel_spread: f64,
    pub microprice: f64,
    pub imb1: f64,
    pub imb3: f64,
    pub imb5: f64,
    pub dt_ms: f64, // Δt от предыдущего снапшота (для стейлнесса)
}

impl Feat {
    #[inline]
    pub fn header_slice() -> &'static [&'static str] {
        &[
            "ts_ms",
            "mid",
            "spread",
            "rel_spread",
            "microprice",
            "imb1",
            "imb3",
            "imb5",
            "dt_ms",
            "label",
        ]
    }
}

#[inline]
fn safe_div(num: f64, den: f64) -> f64 {
    if den.abs() < 1e-12 { 0.0 } else { num / den }
}

/// Вычисляет признаки. `prev_ts_ms` нужен для `dt_ms`.
pub fn compute_features(row: &Snapshot, prev_ts_ms: Option<f64>) -> Feat {
    let mid = 0.5 * (row.ask_price_1 + row.bid_price_1);
    let spread = row.ask_price_1 - row.bid_price_1;
    let rel_spread = safe_div(spread, mid);

    // microprice = (ask1 * bidQty1 + bid1 * askQty1) / (bidQty1 + askQty1)
    let microprice = safe_div(
        row.ask_price_1 * row.bid_qty_1 + row.bid_price_1 * row.ask_qty_1,
        row.bid_qty_1 + row.ask_qty_1,
    );

    // imbalance k = (sum(bidQty_1..k) - sum(askQty_1..k)) / (sum(bid)+sum(ask))
    let b1 = row.bid_qty_1;
    let a1 = row.ask_qty_1;
    let b3 = row.bid_qty_1 + row.bid_qty_2 + row.bid_qty_3;
    let a3 = row.ask_qty_1 + row.ask_qty_2 + row.ask_qty_3;
    let b5 = row.bid_qty_1 + row.bid_qty_2 + row.bid_qty_3 + row.bid_qty_4 + row.bid_qty_5;
    let a5 = row.ask_qty_1 + row.ask_qty_2 + row.ask_qty_3 + row.ask_qty_4 + row.ask_qty_5;

    let imb1 = safe_div(b1 - a1, b1 + a1);
    let imb3 = safe_div(b3 - a3, b3 + a3);
    let imb5 = safe_div(b5 - a5, b5 + a5);

    let dt_ms = prev_ts_ms
        .map(|p| (row.local_timestamp - p).max(0.0))
        .unwrap_or(0.0);

    Feat {
        ts_ms: row.local_timestamp,
        mid,
        spread,
        rel_spread,
        microprice,
        imb1,
        imb3,
        imb5,
        dt_ms,
    }
}
