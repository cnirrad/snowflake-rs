use std::cell::{Cell, RefCell};
use std::{convert::TryInto, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

thread_local! {
    static CLOCK: RefCell<Box<dyn Clock>> = RefCell::new(Box::new(SystemTimeClock(Cell::new(0))));
}

/// A trait for a clock that can get the time (in milliseconds since a given epoch) and 
/// will wait for the next millisecond.
pub trait Clock {
    fn get_time(&self) -> u64;

    fn wait(&self);
}

/// A Clock implementation that uses SystemTime to provide millisecond precision using 2020-01-01 as the epoch.
struct SystemTimeClock(Cell<u64>);

/// A Clock implementation that can be used for tests. Time will not progress unless wait is called, then 1 ms will pass.
struct MockClock(Cell<u64>);

impl Clock for SystemTimeClock {
    fn get_time(&self) -> u64 {
        const EPOCH: u128 = 1577836800000; // 2020-01-01T00:00:00Z
        let tm = SystemTime::now();

        let millis = match tm.duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => panic!("System time is before UNIX_EPOCH"),
        };

        let adj_ms = millis - EPOCH;
        let new_ts = adj_ms.try_into().unwrap();

        // SystemTime::now is not guarenteed to be monotonically increasing, but 
        // the Snowflake requires it to be. If the new ts is not greater than the
        // old, then we just use the old ts.
        if new_ts > self.0.get() {
            self.0.set(new_ts);
        }

        self.0.get()
    }

    fn wait(&self) {
        thread::sleep(Duration::from_millis(1)) 
    }
}

impl Clock for MockClock {
    fn get_time(&self) -> u64 {
        self.0.get()
    }

    fn wait(&self) {
        let ts = self.0.get();
        self.0.set(ts + 1);
    }
}

pub fn get_time() -> u64 {
    CLOCK.with(|c| {
        c.borrow().get_time()
    })
}

pub fn wait() {
    CLOCK.with(|c| {
        c.borrow_mut().wait()
    })
}

/// Set the mock clock to be used.
pub fn setup_mock_clock() {
    CLOCK.with(|c| {
        let ts = c.borrow().get_time();
        let mock = Box::new(MockClock(Cell::new(ts)));
        *c.borrow_mut() = mock;
    })
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell};
    use crate::clock::{Clock, SystemTimeClock};

    #[test]
    fn system_clock_never_goes_backwards() {
        let clock = SystemTimeClock(Cell::new(0));

        let last = clock.get_time();
        for _ in 0..10000 {
            let now = clock.get_time();
            assert!(now >= last);
        }

    }

    
}
