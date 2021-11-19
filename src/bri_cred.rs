use crate::easlib::Credentials;

pub fn get_credentials() -> Credentials {
    Credentials::new(
        "f33c398c-0f77-4351-9f92-1e20fa3fd2f8".to_owned(),
        "e1320735-e174-4150-9edb-b5daf85be6d1".to_owned(),
        "demoAccount".to_owned())
}
