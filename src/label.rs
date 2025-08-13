#[derive(Debug, Clone, Copy)]
pub enum Label {
    Down,
    Flat,
    Up,
}

impl Label {
    pub fn as_str(&self) -> &'static str {
        match self {
            Label::Down => "down",
            Label::Flat => "flat",
            Label::Up => "up",
        }
    }
}

/// eps = max(eps_min, alpha*rel_spread + cost_bp)
#[inline]
pub fn deadband(rel_spread: f64, eps_min: f64, alpha: f64, cost_bp: f64) -> f64 {
    eps_min.max(alpha * rel_spread + cost_bp)
}

/// mid(t) и mid(t+H)
#[inline]
pub fn classify(
    mid_now: f64,
    mid_future: f64,
    rel_spread_now: f64,
    eps_min: f64,
    alpha: f64,
    cost_bp: f64,
) -> Label {
    let r = (mid_future - mid_now) / mid_now.max(1e-12);
    let eps = deadband(rel_spread_now, eps_min, alpha, cost_bp);
    if r > eps {
        Label::Up
    } else if r < -eps {
        Label::Down
    } else {
        Label::Flat
    }
}
