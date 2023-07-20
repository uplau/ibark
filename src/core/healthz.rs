use tokio::runtime::Runtime;

pub fn exec(global: super::cmd::GlobalOptions) -> anyhow::Result<()> {
    let dump_level = global.dump_level;
    let conf = super::conf::Common::from_cmd(global)?;
    if dump_level > 0 {
        return conf.dump();
    }

    Runtime::new()?.block_on(async {
        let ret: anyhow::Result<_> = Ok(());

        super::cli::Output::exec(&format!(
            "Healthz -R {}",
            super::bark::Remote::scheme_host_port(&conf.remote)?
        ));

        let req = reqwest::Client::builder()
            .user_agent(conf.user_agent.as_ref())
            .build()?
            .get(format!("{}/healthz", conf.remote.as_ref()));

        let resp = req.send().await?.text().await?;
        println!("{}", resp);

        ret
    })
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use clap::Parser;

    // #[test]
    fn dump_help() {
        let cli = cli::Main::parse_from(["", "healthz", "--help"]);
    }

    #[test]
    fn test_dump() -> anyhow::Result<()> {
        let cli = cli::Main::parse_from(["", "healthz", "-D"]);
        match cli.command {
            cmd::Commands::Healthz(args) => exec(cli.global)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    #[test]
    fn test_exec() -> anyhow::Result<()> {
        let cli = cli::Main::parse_from(["", "healthz"]);
        match cli.command {
            cmd::Commands::Healthz(args) => exec(cli.global)?,
            _ => unreachable!(),
        }
        Ok(())
    }
}
