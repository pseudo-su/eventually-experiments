#![allow(missing_docs)]

use envconfig::Envconfig;

use demo_store::config::Config;

fn main() -> anyhow::Result<()> {
    let config = Config::init()?;
    smol::run(demo_store::run_projector(config))
}
