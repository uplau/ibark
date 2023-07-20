pub fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or("not found `=`")?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(clap::Args, Debug)]
pub struct GlobalOptions {
    #[arg(
        required = false,
        global = true,
        short = 'C',
        long = "config",
        value_name = "PATHS",
        help = "Specify configuration files."
    )]
    pub config_file_paths: Vec<std::path::PathBuf>,

    #[arg(
        global = true,
        short = 'D',
        long = "dump",
        action = clap::ArgAction::Count,
        default_value_t = 0,
        value_parser = clap::value_parser!(u8).range(0..=2),
        help="Just dump data, will not execute [range: 0..=2]"
    )]
    pub dump_level: u8,

    #[arg(
        global = true,
        short = 'R',
        long,
        help = format!("Specify remote [fallback: {}]", super::conf::fallback_remote())
    )]
    pub remote: Option<String>,

    #[arg(
        global = true,
        short = 'U',
        long,
        value_name = "AGENT",
        help = format!("Specify user-agent [fallback: {}]", super::conf::fallback_user_agent())
    )]
    pub user_agent: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Get remote healthz.
    Healthz(GlobalOptions),

    /// Get remote info.
    Info(GlobalOptions),

    /// [WIP]Ping remote.
    Ping,

    /// [WIP]Send once notification.
    #[command(arg_required_else_help = true)]
    Send,

    /// [WIP]Web interface.
    #[command(arg_required_else_help = true)]
    Server,
}
