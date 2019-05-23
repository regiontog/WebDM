use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[macro_export]
macro_rules! clone {
    ($($id:ident),* in $code:expr) => {{
        $(let $id = $id.clone();)*
        $code
    }}
}

pub(crate) fn hash(x: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn maybe<T, E, F: FnOnce() -> Result<T, E>>(f: F) -> Result<T, E> {
    f()
}