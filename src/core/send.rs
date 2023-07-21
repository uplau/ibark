use indicatif::ProgressBar;
use std::{collections::HashMap, sync::Arc};
use tokio::{runtime::Runtime, sync::Semaphore, task::JoinSet};

#[derive(clap::Args, Debug)]
pub struct SendArgs {
    /// Specify notification contexts
    #[arg(
        required = false,
        short,
        long,
        value_name = "KEYVAL",
        value_hint = clap::ValueHint::Other,
        value_parser = super::cmd::parse_key_val::<String,String>,
        long_help = "Specify notification contexts\n\nSee more at https://github.com/Finb/bark-server/blob/master/docs/API_V2.md#push"
    )]
    pub contexts: Vec<(String, String)>,

    /// Device name from the config file or your full input
    #[arg(
        required = true,
        value_hint = clap::ValueHint::Other,
    )]
    pub devices: Vec<String>,

    #[arg(
        short = 'l',
        long,
        value_name = "LIMIT",
        help = format!("Specify max concurrent tasks [fallback: {}]", super::conf::fallback_limit_conn())
    )]
    pub limit_conn: Option<u16>,
}

#[derive(Default, serde::Deserialize)]
#[serde(default)]
pub struct SendConf<'a> {
    #[serde(borrow, flatten)]
    pub common: super::conf::Common<'a>,

    pub contexts: HashMap<String, String>,
    pub devices: HashMap<String, String>,
    pub limit_conn: u16,
}

impl<'a> std::fmt::Debug for SendConf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f: std::fmt::DebugStruct<'_, '_> =
            f.debug_struct(std::any::type_name::<Self>().split("::").last().unwrap());

        // flatten
        f.field("remote", &self.common.remote);
        f.field("user_agent", &self.common.user_agent);

        f.field("contexts", &self.contexts);
        f.field("devices", &self.devices);
        f.field("limit_conn", &self.limit_conn);

        if self.common._dump_hide {
            f.field("_config", &self.common._config);
        }

        f.finish()
    }
}

impl<'a> SendConf<'a> {
    pub fn builder_default(
        builder: super::conf::SyncBuilder,
    ) -> anyhow::Result<super::conf::SyncBuilder> {
        let default_contexts = crate::hash_map! {
            "a" => "1",
            "level" => "active"
        }
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect::<HashMap<String, String>>();

        Ok(super::conf::Common::builder_default(builder)?
            .set_default("contexts", default_contexts)?
            .set_default("limit_conn", super::conf::fallback_limit_conn())?)
    }

    pub fn dump(mut self) -> anyhow::Result<()> {
        self.common.remote = super::bark::Remote::dump(&self.common.remote)?.into();

        let mut devices = HashMap::with_capacity(self.devices.len());
        for (name, input) in self.devices.into_iter() {
            devices.insert(name, super::bark::Device::dump(&input)?);
        }
        self.devices = devices;

        println!("{:#?}", self);
        Ok(())
    }

    pub fn from_cmd(global: super::cmd::GlobalOptions, args: SendArgs) -> anyhow::Result<Self> {
        let mut fb = if global.config_file_paths.is_empty() {
            super::conf::FileBuilder::with_preset()?
        } else {
            super::conf::FileBuilder::from_cmd_global_options(global.config_file_paths)?
        };

        fb.builder = Self::builder_default(fb.builder)?
            .set_override_option("remote", global.remote)?
            .set_override_option("user_agent", global.user_agent)?
            .set_override(
                "contexts",
                args.contexts.into_iter().collect::<HashMap<_, _>>(),
            )?
            .set_override_option("limit_conn", args.limit_conn)?;

        let mut _self: Self = fb.builder.build()?.try_deserialize()?;
        _self.common._config = super::conf::FileDisplay::new(fb.sources);
        _self.common._dump_hide = global.dump_level >= 2;

        // real
        super::bark::Remote::verify(&_self.common.remote)?;
        _self.contexts = super::bark::Contexts::verify(_self.contexts)?;
        _self.devices = super::bark::Device::find_merge(&_self.devices, args.devices);

        Ok(_self)
    }
}

pub fn exec(global: super::cmd::GlobalOptions, args: SendArgs) -> anyhow::Result<()> {
    let dump_level = global.dump_level;
    let conf = SendConf::from_cmd(global, args)?;
    if dump_level > 0 {
        return conf.dump();
    }

    let (pb_multi, pb_main) = super::cli::Main::create_multi_progress(conf.devices.len() as u64)?;
    let semaphore = Arc::new(Semaphore::new(conf.limit_conn as usize));

    let client = reqwest::Client::builder()
        .user_agent(conf.common.user_agent.as_ref())
        .build()?;

    let mut join_set = JoinSet::new();
    Runtime::new()?.block_on(async {
        let ret: anyhow::Result<_> = Ok(());

        for (index, (name, input)) in conf.devices.into_iter().enumerate() {
            let req = super::bark::Device::new_request(
                &input,
                &client,
                &conf.common.remote,
                &conf.contexts,
            )?;

            let pb_task = pb_multi.insert_before(&pb_main, ProgressBar::new(1));

            join_set.spawn(super::cli::Main::request_handle(
                semaphore.clone(),
                pb_task,
                (index, name),
                req,
            ));
        }

        pb_multi.println(super::cli::Output::exec_string(&format!(
            "Send -R {} -l {}",
            super::bark::Remote::scheme_host_port(&conf.common.remote)?,
            conf.limit_conn
        )))?;

        while let Some(v) = join_set.join_next().await {
            match v {
                Ok(res) => {
                    res?;
                    pb_main.inc(1);
                }
                Err(err) => {
                    return Err(anyhow::anyhow!(err));
                }
            }
        }

        pb_main.finish_with_message("All tasks done");
        println!("\n");
        // super::cli::Main::warn_request_once_err();

        ret
    })
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::util::tests::*;
    use clap::Parser;

    fn test_gen_args(is_err: bool) -> Vec<String> {
        let mut args = vec!["", "send"]
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        args.push(format!("d://{}", random_string(22)));
        args.push(format!(
            "aes://{}:{}@{}/128/cbc/pkcs7",
            random_string(16),
            random_string(16),
            random_string(22)
        ));
        args.push(format!(
            "aes://{}:{}@{}/192/ecb/pkcs7",
            random_string(24),
            random_string(16),
            random_string(22)
        ));
        args.push(format!(
            "aes://{}:{}@{}/256/ecb/pkcs7",
            random_string(32),
            random_string(16),
            random_string(22)
        ));

        if is_err {
            args.push(random_string(random_number(0, 20) as usize));
        }
        args
    }

    // #[test]
    fn dump_help() {
        let cli = cli::Main::parse_from(["", "send", "--help"]);
    }

    #[test]
    fn test_dump() -> anyhow::Result<()> {
        let mut args = test_gen_args(false);
        args.push("-D".to_string());
        dbg!(&args);
        crate::println_dash!(50);

        let cli = cli::Main::parse_from(args);
        match cli.command.unwrap() {
            cmd::Commands::Send(args) => exec(cli.global, args)?,
            _ => unreachable!(),
        }

        Ok(())
    }

    // need workdir/dev/secret.yaml
    #[test]
    fn test_dump_with_secret() -> anyhow::Result<()> {
        let secret = crate::workdir_join!("dev", "secret.yaml");
        if !secret.is_file() {
            cli::Output::warn("not found workdir/dev/secret.yaml, skip");
            return Ok(());
        }

        let cli =
            cli::Main::parse_from(["", "send", "-C", secret.to_str().unwrap(), "d", "aes", "-D"]);
        match cli.command.unwrap() {
            cmd::Commands::Send(args) => exec(cli.global, args)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    // need workdir/dev/secret.yaml
    #[test]
    fn test_exec_with_secret() -> anyhow::Result<()> {
        let secret = crate::workdir_join!("dev", "secret.yaml");
        if !secret.is_file() {
            cli::Output::warn("not found workdir/dev/secret.yaml, skip");
            return Ok(());
        }

        let cli = cli::Main::parse_from(["", "send", "-C", secret.to_str().unwrap(), "d", "aes"]);

        match cli.command.unwrap() {
            cmd::Commands::Send(args) => exec(cli.global, args)?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
