use crate::easlib::Credentials;

pub fn get_credentials() -> Credentials {
    Credentials::new(
        "iiii-iiii".to_owned(),
        "tttt-tttt".to_owned(),
        "my_name".to_owned())
}
