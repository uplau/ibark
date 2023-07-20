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
    /// Specify configuration files.
    #[arg(
        required = false,
        global = true,
        short = 'C',
        long = "config",
        value_name = "PATHS",
        value_hint = clap::ValueHint::AnyPath,
        long_help = "Specify configuration files\n\nUsing this option will override the preset source"
    )]
    pub config_file_paths: Vec<std::path::PathBuf>,

    /// Just dump data, will not execute [range: 0..=2].
    #[arg(
        global = true,
        short = 'D',
        long = "dump",
        action = clap::ArgAction::Count,
        default_value_t = 0,
        value_parser = clap::value_parser!(u8).range(0..=2),
        long_help="Just dump data, will not execute [range: 0..=2]\n\nUsing -D will dump related configuration\nUsing -DD will dump ⬆️  and configuration source"
    )]
    pub dump_level: u8,

    #[arg(
        global = true,
        short = 'R',
        long,
        value_hint = clap::ValueHint::Url,
        help = format!("Specify remote [fallback: {}]", super::conf::fallback_remote())
    )]
    pub remote: Option<String>,

    #[arg(
        global = true,
        short = 'U',
        long,
        value_name = "AGENT",
        value_hint = clap::ValueHint::Other,
        help = format!("Specify user-agent [fallback: {}]", super::conf::fallback_user_agent())
    )]
    pub user_agent: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Get remote healthz.
    Healthz,

    /// Get remote info.
    Info,

    /// Ping remote.
    Ping,

    /// Send once notification.
    #[command(arg_required_else_help = true)]
    Send(super::send::SendArgs),

    /// [WIP]Web interface.
    #[command(arg_required_else_help = true)]
    Server,
}
