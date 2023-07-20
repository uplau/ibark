use clap::Parser;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::{sync::Semaphore, time};

/// iBark is a fully featured Bark cross-platform command line tool written in Rust.
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true, version)]
pub struct Main {
    /// If provided, outputs the completion file for given shell
    #[arg(short, long = "GEN", value_name = "SHELL", value_enum)]
    pub generator: Option<clap_complete::Shell>,

    #[command(flatten)]
    pub global: super::cmd::GlobalOptions,

    #[command(subcommand)]
    pub command: Option<super::cmd::Commands>,
}

impl Main {
    fn request_once_err<'a>() -> MutexGuard<'a, bool> {
        lazy_static! {
            pub static ref REQUEST_ONCE_ERR: Mutex<bool> = Mutex::new(false);
        }
        REQUEST_ONCE_ERR.lock().unwrap()
    }

    pub fn is_request_once_err() -> bool {
        *Self::request_once_err()
    }

    pub fn set_request_once_err(is_err: bool) {
        let mut ch = Self::request_once_err();
        *ch = is_err;
    }

    pub fn warn_request_once_err() {
        if Self::is_request_once_err() {
            println!("\n\n");
            Output::warn("At least one request error occurred");
        }
    }

    pub fn create_multi_progress(tasks_count: u64) -> anyhow::Result<(MultiProgress, ProgressBar)> {
        let pb_multi = MultiProgress::new();

        let pb_main = pb_multi.add(ProgressBar::new(tasks_count));
        let pb_main_msg_width = match (tasks_count as f64).log10() as u32 + 1 {
            v if v >= 10 => 0,
            v => 9 - v,
        };
        let pb_main_style_str =
            format!("{{spinner:.green}} {{bar:30.cyan/blue}} {{pos}}/{{len:{pb_main_msg_width}}} {{wide_msg}} {{elapsed:.yellow}}");

        pb_main.set_style(
            ProgressStyle::with_template(pb_main_style_str.as_str())?.progress_chars("##-"),
        );

        pb_main.tick();
        Ok((pb_multi, pb_main))
    }

    pub async fn request_handle(
        semaphore: Arc<Semaphore>,
        pb_task: ProgressBar,
        iname: (usize, String),
        req: reqwest::RequestBuilder,
    ) -> anyhow::Result<()> {
        let _permit = semaphore.acquire().await?;

        pb_task.set_style(ProgressStyle::with_template(
            "{spinner:.green} {prefix:30} {wide_msg} {elapsed:.yellow}",
        )?);

        pb_task.set_prefix(format!("#{:<3} {}", iname.0, iname.1));
        pb_task.set_message("Sending");

        // #[cfg(test)]
        // {
        //     use crate::util::tests::*;
        //     let delay = random_number(1, 8);
        //     time::sleep(time::Duration::from_secs(2)).await;
        //     pb_task.set_message(format!("TDelay {delay}s"));
        //     for _ in 0..delay {
        //         time::sleep(time::Duration::from_secs(1)).await;
        //     }
        //     pb_task.set_message("TStart");
        // }

        tokio::select! {
            biased;
            v = req.send() =>{
                match v {
                    Ok(resp)=>{
                        let status = resp.status();
                        let title = if status == 200 {
                            "Success".bold().blue()
                        } else {
                            Main::set_request_once_err(true);
                            "Failed".bold().red()
                        };
                        pb_task.set_message(format!("{:10} Status {status}", title));
                    },
                    Err(err)=>{
                        Main::set_request_once_err(true);
                        let title = "Error".bold().red();
                        pb_task.set_message(format!("{:10} Status {err}", title));
                    }
                }
            },
            _ = async{
                loop{
                    pb_task.tick();
                    time::sleep(time::Duration::from_millis(50)).await;
                }
            } => {}
        };

        pb_task.finish();
        Ok(())
    }

    pub fn output_completions<S: clap_complete::Generator>(shell: S, command: &mut clap::Command) {
        use clap_complete::generate;
        generate(
            shell,
            command,
            command.get_name().to_string(),
            &mut std::io::stdout(),
        );
    }
}

pub struct Output;
impl Output {
    pub fn info() {
        todo!()
    }

    pub fn exec(s: &str) {
        println!("{}", Self::exec_string(s));
    }

    pub fn exec_string(s: &str) -> String {
        format!("{}{}", "[+] ".bold().green(), s.bold().green())
    }

    pub fn warn(s: &str) {
        println!("{}{}", "WARN: ".bold().yellow(), s.bold().yellow());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::tests::*;
    use tokio::{runtime::Runtime, task};

    // #[test]
    fn dump_help() {
        let cli = Main::parse_from(["", "--help"]);
        dbg!(&cli);
    }

    #[test]
    fn test_multi_request() -> anyhow::Result<()> {
        let client = reqwest::Client::builder().build()?;

        let create_request_tasks = |count: usize| {
            let mut vec = Vec::with_capacity(count);
            vec.push(client.post("https://httpbin.org/status/301"));
            vec.push(client.post("https://httpbin.org/status/200"));
            vec.push(client.post("https://httpbin.org/status/302"));
            vec.push(client.post("https://httpbin.org/status/404"));
            vec.push(client.post("https://httpbin.org/status/500"));
            let count = count - 5;

            for index in 0..count {
                let status_code = random_number(200, 500);
                let url = format!("https://httpbin.org/status/{status_code}");
                vec.push(client.post(url));
            }
            vec
        };

        let vec = create_request_tasks(11);
        // dbg!(&vec);
        // crate::println_dash!(50);
        ////////////////////////////////////////////////
        let (pb_multi, pb_main) = Main::create_multi_progress(vec.len() as u64)?;
        let limit_conn = 10;
        let semaphore = Arc::new(Semaphore::new(limit_conn));
        pb_multi.println(Output::exec_string(&format!("Test -l {}", limit_conn)));

        let mut join_set = task::JoinSet::new();
        Runtime::new()?.block_on(async {
            let ret: anyhow::Result<_> = Ok(());

            for (index, req) in vec.into_iter().enumerate() {
                let name = crate::util::hash_hex_string(index);
                let pb_task = pb_multi.insert_before(&pb_main, ProgressBar::new(1));

                join_set.spawn(Main::request_handle(
                    semaphore.clone(),
                    pb_task,
                    (index, name),
                    req,
                ));
            }

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
            Main::warn_request_once_err();

            ret
        })
    }
}
