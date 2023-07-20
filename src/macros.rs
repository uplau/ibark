#[macro_export]
macro_rules! named {
    () => {
        "ibark"
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! workdir {
    () => {
        env!("CARGO_MANIFEST_DIR")
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! workdir_join {
    () => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
    };
    ($($expr: expr),*) => {
        {
            let mut path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
            $(path.push($expr);)*
            path
        }
    };
}

#[macro_export]
macro_rules! pkg_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

#[macro_export]
macro_rules! pkg_ver {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

#[macro_export]
macro_rules! user_agent {
    () => {
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
    };
}

#[macro_export]
macro_rules! hash_map {
    {$($k: expr => $v: expr),* $(,)?} => {
            std::collections::HashMap::from([$(($k, $v),)*])
    };
}

#[macro_export]
macro_rules! btree_map {
    {$($k: expr => $v: expr),* $(,)?} => {
        std::collections::BTreeMap::from([$(($k, $v),)*])
    };
}

#[macro_export]
macro_rules! vec_map {
    {$($k: expr => $v: expr),* $(,)?} => {
        vec!($(($k, $v),)*)
    };
}

#[macro_export]
macro_rules! println_dash {
    ($expr: expr) => {
        println!("{}", "-".repeat($expr));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;

    #[test]
    fn test_macros() {
        dbg!(named!());
        dbg!(workdir!());
        dbg!(workdir_join!());
        dbg!(workdir_join!("hello", "world"));
        dbg!(pkg_name!());
        dbg!(pkg_ver!());
        dbg!(user_agent!());
        dbg!(hash_map! {
            util::hash_hex_string(util::tests::random_string(util::tests::random_number(0, 32) as usize)) => util::tests::random_string(util::tests::random_number(0, 16) as usize),
            "hello".into() => "hash_map".into()
        });
        dbg!(btree_map! {
            util::hash_hex_string(util::tests::random_string(util::tests::random_number(0, 32) as usize)) => util::tests::random_string(util::tests::random_number(0, 16) as usize),
            "hello".into() => "btree_map".into()
        });
        dbg!(vec_map! {
            util::hash_hex_string(util::tests::random_string(util::tests::random_number(0, 32) as usize)) => util::tests::random_string(util::tests::random_number(0, 16) as usize),
            "hello".into() => "vec_map".into()
        });
    }
}
