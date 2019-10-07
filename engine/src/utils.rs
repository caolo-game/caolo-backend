#![allow(unused)]

#[macro_export(local_inner_macros)]
macro_rules! profile {
    ($name: expr) => {
        #[cfg(feature = "profile")]
        let _profile = {
            use crate::utils::profiler::Profiler;
            Profiler::new($name)
        };
    };
}

pub mod profiler {
    use std::time::Instant;

    pub struct Profiler<'a> {
        start: Instant,
        name: &'a str,
    }

    impl<'a> Profiler<'a> {
        pub fn new(name: &'a str) -> Self {
            let start = Instant::now();
            Self { name, start }
        }
    }

    impl<'a> Drop for Profiler<'a> {
        fn drop(&mut self) {
            let end = Instant::now();
            let dur = end - self.start;
            let mil = dur.as_millis();

            debug!("{} has been completed in {} ms", self.name, mil);
        }
    }
}
