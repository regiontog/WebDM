use serde::{Deserialize, Serialize};
#[derive(Deserialize, Debug)]
pub(crate) struct Callback<T> {
    pub(crate) id: u64,
    pub(crate) data: T,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Login {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Deserialize, Debug)]
pub(crate) enum Exit {}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub(crate) struct Session {
    pub(crate) name: String,
    pub(crate) key: u64,
    pub(crate) comment: String,
}
