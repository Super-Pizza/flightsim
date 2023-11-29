#[macro_export]
macro_rules! span {
    ($loc:expr) => {
        profiling::Client::running().unwrap().span($loc, 0)
    };
}
