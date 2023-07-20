use core::cell::Cell;
use core::marker::PhantomData;

/// Marker type that implements `!Send` and `!Sync`.
/// Workaround for unimplemented negative trait impls.
pub type PhantomUnsend = PhantomData<*const ()>;

/// Marker type that implements `!Sync`.
/// Workaround for unimplemented negative trait impls.
pub type PhantomUnsync = PhantomData<Cell<()>>;
