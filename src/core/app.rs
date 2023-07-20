pub fn start() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    use clap::Parser;
    let cli = super::cli::Main::parse();

    match cli.command {
        super::cmd::Commands::Healthz => super::misc::exec(cli.global, "healthz"),
        super::cmd::Commands::Info => super::misc::exec(cli.global, "info"),
        super::cmd::Commands::Ping => super::misc::exec(cli.global, "ping"),
        super::cmd::Commands::Send(args) => super::send::exec(cli.global, args),
        super::cmd::Commands::Server => todo!(),
    }
}
