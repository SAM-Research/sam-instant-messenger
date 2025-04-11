use std::sync::atomic::{AtomicU16, Ordering};

static BASE_PORT: u16 = 3080;
static PORT_ORACLE: AtomicU16 = AtomicU16::new(BASE_PORT);

/// Get the next available port from the oracle.
pub fn get_next_port() -> u16 {
    loop {
        let current = PORT_ORACLE.load(Ordering::SeqCst);

        let next_port = if current % 1000 == 89 {
            current + 991
        } else {
            current + 1
        };

        if PORT_ORACLE
            .compare_exchange(current, next_port, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return current;
        }
    }
}
