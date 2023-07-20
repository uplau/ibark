#![cfg_attr(debug_assertions, allow(unused, dead_code))]

mod core;
mod macros;
mod util;

fn main() -> anyhow::Result<()> {
    crate::core::app::start()
}
