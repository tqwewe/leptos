//! Computed reactive values that derive from other reactive values.

mod arc_memo;
mod async_derived;
mod inner;
mod memo;
mod selector;
use crate::{
    prelude::*,
    signal::RwSignal,
    wrappers::{
        read::Signal,
        write::{IntoSignalSetter, SignalSetter},
    },
};
pub use arc_memo::*;
pub use async_derived::*;
pub use memo::*;
pub use selector::*;

/// Derives a reactive slice of an [`RwSignal`](crate::signal::RwSignal).
///
/// Slices have the same guarantees as [`Memo`s](crate::computed::Memo):
/// they only emit their value when it has actually been changed.
///
/// Slices need a getter and a setter, and you must make sure that
/// the setter and getter only touch their respective field and nothing else.
/// They optimally should not have any side effects.
///
/// You can use slices whenever you want to react to only parts
/// of a bigger signal. The prime example would be state management,
/// where you want all state variables grouped together, but also need
/// fine-grained signals for each or some of these variables.
/// In the example below, setting an auth token will only trigger
/// the token signal, but none of the other derived signals.
/// ```
/// # use reactive_graph::prelude::*;
/// # use reactive_graph::effect::Effect;
/// # use reactive_graph::signal::RwSignal;
/// # use reactive_graph::computed::*;
///
/// // some global state with independent fields
/// #[derive(Default, Clone, Debug)]
/// struct GlobalState {
///     count: u32,
///     name: String,
/// }
///
/// let state = RwSignal::new(GlobalState::default());
///
/// // `create_slice` lets us create a "lens" into the data
/// let (count, set_count) = create_slice(
///     // we take a slice *from* `state`
///     state,
///     // our getter returns a "slice" of the data
///     |state| state.count,
///     // our setter describes how to mutate that slice, given a new value
///     |state, n| state.count = n,
/// );
///
/// // this slice is completely independent of the `count` slice
/// // neither of them will cause the other to rerun
/// let (name, set_name) = create_slice(
///     // we take a slice *from* `state`
///     state,
///     // our getter returns a "slice" of the data
///     |state| state.name.clone(),
///     // our setter describes how to mutate that slice, given a new value
///     |state, n| state.name = n,
/// );
///
/// Effect::new(move |_| {
///     // note: in the browser, use leptos::log! instead
///     println!("name is {}", name.get());
/// });
/// Effect::new(move |_| {
///     println!("count is {}", count.get());
/// });
///
/// // setting count only causes count to log, not name
/// set_count.set(42);
///
/// // setting name only causes name to log, not count
/// set_name.set("Bob".into());
/// ```
#[track_caller]
pub fn create_slice<T, O, S>(
    signal: RwSignal<T>,
    getter: impl Fn(&T) -> O + Copy + Send + Sync + 'static,
    setter: impl Fn(&mut T, S) + Copy + Send + Sync + 'static,
) -> (Signal<O>, SignalSetter<S>)
where
    T: Send + Sync + 'static,
    O: PartialEq + Send + Sync + 'static,
{
    (
        create_read_slice(signal, getter),
        create_write_slice(signal, setter),
    )
}

/// Takes a memoized, read-only slice of a signal. This is equivalent to the
/// read-only half of [`create_slice`].
#[track_caller]
pub fn create_read_slice<T, O>(
    signal: RwSignal<T>,
    getter: impl Fn(&T) -> O + Copy + Send + Sync + 'static,
) -> Signal<O>
where
    T: Send + Sync + 'static,
    O: PartialEq + Send + Sync + 'static,
{
    Memo::new(move |_| signal.with(getter)).into()
}

/// Creates a setter to access one slice of a signal. This is equivalent to the
/// write-only half of [`create_slice`].
#[track_caller]
pub fn create_write_slice<T, O>(
    signal: RwSignal<T>,
    setter: impl Fn(&mut T, O) + Copy + Send + Sync + 'static,
) -> SignalSetter<O>
where
    T: Send + Sync + 'static,
{
    let setter = move |value| signal.update(|x| setter(x, value));
    setter.into_signal_setter()
}
