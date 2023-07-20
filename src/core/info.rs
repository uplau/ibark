use tokio::runtime::Runtime;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct InfoResponse<'a> {
    #[serde(borrow)]
    version: &'a str,
}

pub fn exec(global: super::cmd::GlobalOptions) -> anyhow::Result<()> {
    let dump_level = global.dump_level;
    let conf = super::conf::Common::from_cmd(global)?;
    if dump_level > 0 {
        return conf.dump();
    }

    Runtime::new()?.block_on(async {
        let ret: anyhow::Result<_> = Ok(());

        super::cli::Output::exec(&format!(
            "Info -R {}",
            super::bark::Remote::scheme_host_port(&conf.remote)?
        ));

        let req = reqwest::Client::builder()
            .user_agent(conf.user_agent.as_ref())
            .build()?
            .get(format!("{}/info", conf.remote.as_ref()));

        let resp = req.send().await?.text().await?;
        match serde_json::from_str::<InfoResponse>(&resp) {
            Ok(v) => {
                println!("{:#?}", v);
            }
            Err(_) => {
                println!("{}", resp);
            }
        }

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
        let cli = cli::Main::parse_from(["", "info", "--help"]);
    }

    #[test]
    fn test_dump() -> anyhow::Result<()> {
        let cli = cli::Main::parse_from(["", "info", "-D"]);
        match cli.command {
            cmd::Commands::Info(args) => exec(cli.global)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    #[test]
    fn test_exec() -> anyhow::Result<()> {
        let cli = cli::Main::parse_from(["", "info"]);
        match cli.command {
            cmd::Commands::Info(args) => exec(cli.global)?,
            _ => unreachable!(),
        }
        Ok(())
    }
}
