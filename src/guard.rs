//! Anti-rebond : évite de déclencher / couper le flood sur une détection isolée.

use std::time::{Duration, Instant};

pub const AWAY_DELAY: Duration = Duration::from_millis(900);
pub const BACK_DELAY: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct AwayGuard {
    away_since: Option<Instant>,
    back_since: Option<Instant>,
    flooding: bool,
}

impl AwayGuard {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// `looking` : l'utilisateur est face à l'écran. Renvoie l'état du flood.
    pub fn update(&mut self, looking: bool, now: Instant) -> bool {
        if looking {
            self.away_since = None;
            let since = *self.back_since.get_or_insert(now);
            if self.flooding && now.duration_since(since) >= BACK_DELAY {
                self.flooding = false;
            }
        } else {
            self.back_since = None;
            let since = *self.away_since.get_or_insert(now);
            if !self.flooding && now.duration_since(since) >= AWAY_DELAY {
                self.flooding = true;
            }
        }
        self.flooding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floods_after_delay_and_stops_after_return() {
        let t0 = Instant::now();
        let mut g = AwayGuard::default();
        assert!(!g.update(false, t0));
        assert!(!g.update(false, t0 + Duration::from_millis(500)));
        assert!(g.update(false, t0 + AWAY_DELAY));
        assert!(g.update(true, t0 + Duration::from_secs(2)));
        assert!(!g.update(true, t0 + Duration::from_secs(2) + BACK_DELAY));
    }

    #[test]
    fn short_glitch_does_not_flood() {
        let t0 = Instant::now();
        let mut g = AwayGuard::default();
        g.update(false, t0);
        g.update(true, t0 + Duration::from_millis(400));
        assert!(!g.update(false, t0 + Duration::from_millis(1000)));
    }
}
