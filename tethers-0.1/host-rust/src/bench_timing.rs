//! Benchmark-only stage timing sink (PF1 Part D).
//!
//! When the `bench-timing` cargo feature is enabled, [`timed`] records the
//! elapsed duration of a closure under a `&'static str` stage label into a
//! process-local thread-local accumulator, which the benchmark binary reads
//! with [`snapshot`] and [`reset`].
//!
//! When the feature is disabled every function compiles to a zero-cost no-op
//! passthrough, so production builds are unaffected: no clock reads, no
//! allocation, no behaviour, ordering, persistence, or error-handling change.
//!
//! This is observation-only instrumentation for the benchmark path. It never
//! alters a decision, skips work, caches anything, or changes ordering.

#[cfg(feature = "bench-timing")]
mod impl_ {
    use std::cell::RefCell;
    use std::time::Instant;

    thread_local! {
        static STAGES: RefCell<Vec<(&'static str, u128)>> = const { RefCell::new(Vec::new()) };
    }

    /// Run `f`, record its duration under `stage` (microseconds), return `f`'s value.
    pub fn timed<R>(stage: &'static str, f: impl FnOnce() -> R) -> R {
        let t0 = Instant::now();
        let result = f();
        let elapsed = t0.elapsed();
        STAGES.with(|stages| stages.borrow_mut().push((stage, elapsed.as_micros())));
        result
    }

    /// Raw read of the accumulated stage samples since the last [`reset`].
    pub fn snapshot() -> Vec<(&'static str, u128)> {
        STAGES.with(|stages| stages.borrow().clone())
    }

    /// Drop accumulated samples.
    pub fn reset() {
        STAGES.with(|stages| stages.borrow_mut().clear());
    }
}

#[cfg(not(feature = "bench-timing"))]
mod impl_ {
    /// Zero-cost passthrough when the feature is disabled.
    pub fn timed<R>(_stage: &'static str, f: impl FnOnce() -> R) -> R {
        f()
    }

    /// Empty snapshot when the feature is disabled.
    pub fn snapshot() -> Vec<(&'static str, u128)> {
        Vec::new()
    }

    /// No-op when the feature is disabled.
    pub fn reset() {}
}

pub use impl_::{reset, snapshot, timed};
