pub fn start() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    use clap::{CommandFactory, Parser};
    let cli = super::cli::Main::parse();

    if let Some(shell) = cli.generator {
        let mut command = super::cli::Main::command();
        super::cli::Main::output_completions(shell, &mut command);
        return Ok(());
    }

    let mut is_use_request_once_err = false;
    if let Some(command) = cli.command {
        match command {
            super::cmd::Commands::Healthz => super::misc::exec(cli.global, "healthz")?,
            super::cmd::Commands::Info => super::misc::exec(cli.global, "info")?,
            super::cmd::Commands::Ping => super::misc::exec(cli.global, "ping")?,
            super::cmd::Commands::Send(args) => {
                is_use_request_once_err = true;
                super::send::exec(cli.global, args)?
            }
            super::cmd::Commands::Server => todo!(),
        }
    }

    if is_use_request_once_err && super::cli::Main::is_request_once_err() {
        use anyhow::anyhow;
        return Err(anyhow!("at least one request error occurred"));
    }

    Ok(())
}
