/// Non-atomic adapter types. Only safe for single threaded applications.
pub mod unsync;

/// Atomic adapter types. Safe for both single and multi threaded
/// applications.
///
/// Atomics have significant performance overhead over their [`unsync`]
/// counterparts.
pub mod sync;
