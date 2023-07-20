use tokio::runtime::Runtime;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct InfoResponse<'a> {
    #[serde(borrow)]
    version: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct PingResponse<'a> {
    code: u16,
    #[serde(borrow)]
    message: &'a str,
    timestamp: u64,
}

pub fn exec(global: super::cmd::GlobalOptions, name: &str) -> anyhow::Result<()> {
    match name {
        "healthz" | "info" | "ping" => {}
        _ => unreachable!(),
    }

    let dump_level = global.dump_level;
    let conf = super::conf::Common::from_cmd(global)?;
    if dump_level > 0 {
        return conf.dump();
    }

    Runtime::new()?.block_on(async {
        let ret: anyhow::Result<_> = Ok(());

        super::cli::Output::exec(&format!(
            "{} -R {}",
            name[0..1].to_uppercase() + &name[1..],
            super::bark::Remote::scheme_host_port(&conf.remote)?
        ));

        let req = reqwest::Client::builder()
            .user_agent(conf.user_agent.as_ref())
            .build()?
            .get(format!("{}/{name}", conf.remote.as_ref()));

        let resp = req.send().await?.text().await?;

        match name {
            "healthz" => {
                println!("{}", resp);
            }
            "info" => match serde_json::from_str::<InfoResponse>(&resp) {
                Ok(v) => {
                    println!("{:#?}", v);
                }
                Err(_) => {
                    println!("{}", resp);
                }
            },
            "ping" => match serde_json::from_str::<PingResponse>(&resp) {
                Ok(v) => {
                    println!("{:#?}", v);
                }
                Err(_) => {
                    println!("{}", resp);
                }
            },
            _ => unreachable!(),
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
        // healthz info ping
        let cli = cli::Main::parse_from(["", "healthz", "--help"]);
    }

    #[test]
    fn test_dump() -> anyhow::Result<()> {
        let miscs = ["healthz", "info", "ping"];

        for misc in miscs.iter() {
            let cli = cli::Main::parse_from(["", misc, "-D"]);
            match cli.command {
                cmd::Commands::Healthz => misc::exec(cli.global, "healthz")?,
                cmd::Commands::Info => misc::exec(cli.global, "info")?,
                cmd::Commands::Ping => misc::exec(cli.global, "ping")?,
                _ => unreachable!(),
            }
            crate::println_dash!(50);
        }

        Ok(())
    }

    #[test]
    fn test_exec() -> anyhow::Result<()> {
        let miscs = ["healthz", "info", "ping"];

        for misc in miscs.iter() {
            let cli = cli::Main::parse_from(["", misc]);
            match cli.command {
                cmd::Commands::Healthz => misc::exec(cli.global, "healthz")?,
                cmd::Commands::Info => misc::exec(cli.global, "info")?,
                cmd::Commands::Ping => misc::exec(cli.global, "ping")?,
                _ => unreachable!(),
            }
            crate::println_dash!(50);
        }

        Ok(())
    }
}
