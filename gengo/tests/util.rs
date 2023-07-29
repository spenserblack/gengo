#[macro_export]
macro_rules! fixture_str {
    ($name:literal) => {
        include_str!(concat!("./fixtures/", $name))
    };
}

#[macro_export]
macro_rules! fixture_bytes {
    ($name:literal) => {
        include_bytes!(concat!("./fixtures/", $name))
    };
}
