/// If `profile` feature is enabled, records high-level profiling information to `profile.csv`.
/// Recording is done via a thread-local buffer and dedicated file writing thread, in an attempt to
/// mitigate overhead.
///
#[macro_export(local_inner_macros)]
macro_rules! profile {
    ($name: expr) => {
        #[cfg(feature = "profile")]
        cao_profile::profile!($name)
    };
}
