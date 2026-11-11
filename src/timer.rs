//! Logique du Pomodoro, indépendante de l'interface.

use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Work,
    Break,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Work => "TRAVAIL",
            Phase::Break => "PAUSE",
        }
    }

    fn other(self) -> Phase {
        match self {
            Phase::Work => Phase::Break,
            Phase::Break => Phase::Work,
        }
    }
}

pub struct Pomodoro {
    pub phase: Phase,
    pub work_min: u32,
    pub break_min: u32,
    pub running: bool,
    remaining: Duration,
}

impl Pomodoro {
    pub fn new(work_min: u32, break_min: u32) -> Self {
        Self {
            phase: Phase::Work,
            work_min,
            break_min,
            running: false,
            remaining: Duration::from_secs(u64::from(work_min) * 60),
        }
    }

    fn phase_len(&self, phase: Phase) -> Duration {
        let min = match phase {
            Phase::Work => self.work_min,
            Phase::Break => self.break_min,
        };
        Duration::from_secs(u64::from(min.max(1)) * 60)
    }

    /// Secondes restantes, arrondies au supérieur (25:00 affiché dès le départ).
    pub fn remaining_secs(&self) -> u64 {
        self.remaining.as_millis().div_ceil(1000) as u64
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    /// Remet une session de travail complète.
    pub fn reset(&mut self) {
        self.running = false;
        self.phase = Phase::Work;
        self.remaining = self.phase_len(Phase::Work);
    }

    /// Reapplique les durées réglées à la phase courante (uniquement à l'arrêt).
    pub fn apply_settings(&mut self) {
        if !self.running {
            self.remaining = self.phase_len(self.phase);
        }
    }

    /// Fait avancer le temps. Renvoie `true` si la phase vient de changer.
    pub fn advance(&mut self, dt: Duration) -> bool {
        if !self.running {
            return false;
        }
        if dt < self.remaining {
            self.remaining -= dt;
            return false;
        }
        self.phase = self.phase.other();
        self.remaining = self.phase_len(self.phase);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_25_minutes() {
        let p = Pomodoro::new(25, 5);
        assert_eq!(p.remaining_secs(), 25 * 60);
    }

    #[test]
    fn does_not_advance_when_paused() {
        let mut p = Pomodoro::new(25, 5);
        assert!(!p.advance(Duration::from_secs(60)));
        assert_eq!(p.remaining_secs(), 25 * 60);
    }

    #[test]
    fn switches_to_break_then_back() {
        let mut p = Pomodoro::new(25, 5);
        p.toggle();
        assert!(!p.advance(Duration::from_secs(25 * 60 - 1)));
        assert_eq!(p.remaining_secs(), 1);
        assert!(p.advance(Duration::from_secs(1)));
        assert_eq!(p.phase, Phase::Break);
        assert_eq!(p.remaining_secs(), 5 * 60);
        assert!(p.advance(Duration::from_secs(5 * 60)));
        assert_eq!(p.phase, Phase::Work);
    }

    #[test]
    fn reset_restores_work_phase() {
        let mut p = Pomodoro::new(10, 2);
        p.toggle();
        p.advance(Duration::from_secs(10 * 60));
        p.reset();
        assert_eq!((p.phase, p.running, p.remaining_secs()), (Phase::Work, false, 600));
    }
}
