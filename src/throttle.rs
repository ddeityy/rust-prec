use std::time::{Duration, Instant};

pub struct Throttler {
    interval: Duration,
    last: Instant,
}

impl Throttler {
    pub fn new(interval: Duration) -> Self {
        Throttler {
            interval,
            last: Instant::now() - interval,
        }
    }

    pub fn debounce<T>(&mut self, val: T) -> Option<T> {
        if self.last.elapsed() > self.interval {
            self.last = Instant::now();
            Some(val)
        } else {
            None
        }
    }
}

#[test]
fn test_throttle() {
    use std::thread::sleep;

    let mut debouncer = Throttler::new(Duration::from_millis(10));
    assert!(debouncer.debounce(1).is_some());
    assert!(debouncer.debounce(1).is_none());
    sleep(Duration::from_millis(15));
    assert!(debouncer.debounce(1).is_some());
}
