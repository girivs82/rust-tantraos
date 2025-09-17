//! TantraOS time implementation

#![allow(dead_code)]

use crate::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant {
    t: u64,
}

impl Instant {
    pub fn now() -> Instant {
        Instant {
            // TODO: Get actual timestamp from TantraOS kernel
            t: 0
        }
    }

    pub fn checked_sub(&self, other: &Instant) -> Option<Duration> {
        if self.t >= other.t {
            Some(Duration::from_nanos(self.t - other.t))
        } else {
            None
        }
    }

    pub fn checked_add(&self, duration: &Duration) -> Option<Instant> {
        self.t.checked_add(duration.as_nanos() as u64).map(|t| Instant { t })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        self.t.checked_sub(other.as_nanos() as u64).map(|t| Instant { t })
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.checked_sub(other)
    }

    pub fn checked_add_duration(&self, duration: &Duration) -> Option<Instant> {
        self.checked_add(duration)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime {
    t: u64,
}

pub const UNIX_EPOCH: SystemTime = SystemTime { t: 0 };

impl SystemTime {
    pub fn now() -> SystemTime {
        SystemTime {
            // TODO: Get actual system time from TantraOS kernel
            t: 0
        }
    }

    pub fn checked_add(&self, duration: &Duration) -> Option<SystemTime> {
        self.t.checked_add(duration.as_nanos() as u64).map(|t| SystemTime { t })
    }

    pub fn checked_sub(&self, duration: &Duration) -> Option<SystemTime> {
        self.t.checked_sub(duration.as_nanos() as u64).map(|t| SystemTime { t })
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        if self.t >= other.t {
            Ok(Duration::from_nanos(self.t - other.t))
        } else {
            Err(Duration::from_nanos(other.t - self.t))
        }
    }

    pub fn checked_add_duration(&self, duration: &Duration) -> Option<SystemTime> {
        self.checked_add(duration)
    }

    pub fn checked_sub_duration(&self, duration: &Duration) -> Option<SystemTime> {
        self.checked_sub(duration)
    }
}

pub fn sleep(dur: Duration) {
    let _ = dur;
    // TODO: Implement sleep via TantraOS scheduler
    // For now, busy wait (not ideal but minimal implementation)
    let start = Instant::now();
    while start.checked_add(&dur).map(|end| Instant::now() < end).unwrap_or(false) {
        // Busy wait
    }
}