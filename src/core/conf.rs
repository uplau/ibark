use anyhow::Context;
use config::{builder::DefaultState, Config, ConfigBuilder, File, FileFormat};
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};

pub type SyncBuilder = ConfigBuilder<DefaultState>;

#[inline]
pub fn fallback_limit_conn() -> u16 {
    10
}

#[inline]
pub fn fallback_remote<'a>() -> &'a str {
    "https://api.day.app"
}

#[inline]
pub fn fallback_user_agent<'a>() -> &'a str {
    crate::user_agent!()
}

pub struct FileSource {
    pub alias: String,
    pub using: Vec<FileFormat>,
    pub abs: String,
    pub required: bool,
    _is_input: bool,
}

impl std::fmt::Debug for FileSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let abs_name = if !self._is_input {
            "abs_base"
        } else {
            "abs_file"
        };

        f.debug_struct(std::any::type_name::<Self>().split("::").last().unwrap())
            .field("alias", &self.alias)
            .field("using", &format_args!("{:?}", self.using))
            .field(abs_name, &self.abs)
            .field("required", &self.required)
            .finish()
    }
}

impl FileSource {
    pub fn new<T1, T2>(alias: T1, abs: T2, required: bool, is_input: bool) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            alias: alias.into(),
            using: Vec::with_capacity(FileBuilder::file_format_len()),
            abs: abs.into(),
            required,
            _is_input: is_input,
        }
    }

    pub fn preset_etc(named: &str, required: bool) -> anyhow::Result<Self> {
        let alias = "global_dir_config";

        #[cfg(target_os = "windows")]
        {
            // %ProgramData% - guess requires administrator privileges
            Ok(Self::new(
                alias,
                directories::UserDirs::new()
                    .unwrap()
                    .public_dir()
                    .unwrap()
                    .join(named)
                    .join("preset")
                    .to_str()
                    .unwrap(),
                required,
                false,
            ))
        }

        #[cfg(unix)]
        {
            Ok(Self::new(
                alias,
                format!("/etc/{named}/preset"),
                required,
                false,
            ))
        }
    }

    pub fn preset_home(named: &str, required: bool) -> anyhow::Result<Self> {
        use directories::BaseDirs;

        Ok(Self::new(
            "user_dir_config",
            BaseDirs::new()
                .unwrap()
                .home_dir()
                .join(".config")
                .join(named)
                .join("preset")
                .to_str()
                .unwrap(),
            required,
            false,
        ))
    }

    pub fn preset_pwd(required: bool) -> anyhow::Result<Self> {
        Ok(Self::new(
            "current_dir_config",
            Path::new(".")
                .canonicalize()?
                .join("preset")
                .to_str()
                .unwrap(),
            required,
            false,
        ))
    }

    pub fn preset_apps() -> anyhow::Result<Vec<Self>> {
        Ok(vec![
            Self::preset_etc(crate::named!(), false)?,
            Self::preset_home(crate::named!(), false)?,
            Self::preset_pwd(false)?,
        ])
    }
}

#[derive(Debug)]
pub struct FileBuilder {
    pub builder: SyncBuilder,
    pub sources: Vec<FileSource>,
}

impl FileBuilder {
    fn lazy_using(sources: &mut [FileSource]) {
        lazy_static! {
            static ref IS_LAZY_USING: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
        }

        let mut is_lazy_using = IS_LAZY_USING.lock().unwrap();
        if *is_lazy_using {
            println!("warn: is_lazy_using");
            return;
        }

        for s in sources.iter_mut() {
            if s.abs.is_empty() {
                continue;
            }
            let p = Path::new(&s.abs);

            if !s._is_input {
                let mut try_push = |ext: &str, fmt: FileFormat| {
                    if p.with_extension(ext).is_file() && s.using.len() < Self::file_format_len() {
                        s.using.push(fmt);
                    }
                };

                try_push("ini", FileFormat::Ini);
                try_push("json", FileFormat::Json);
                try_push("json5", FileFormat::Json5);
                try_push("toml", FileFormat::Toml);
                try_push("yaml", FileFormat::Yaml);
                try_push("yml", FileFormat::Yaml);
            } else {
                let ext = p.extension().unwrap().to_str().unwrap().to_lowercase();
                match ext.as_str() {
                    "ini" => {
                        s.using.push(FileFormat::Ini);
                        continue;
                    }
                    "json" => {
                        s.using.push(FileFormat::Json);
                        continue;
                    }
                    "json5" => {
                        s.using.push(FileFormat::Json5);
                        continue;
                    }
                    "toml" => {
                        s.using.push(FileFormat::Toml);
                        continue;
                    }
                    "yaml" | "yml" => {
                        s.using.push(FileFormat::Yaml);
                        continue;
                    }
                    _ => unreachable!(),
                }
            }
        }

        *is_lazy_using = true;
    }

    #[inline]
    pub fn seq_file_format<'a>() -> &'a [FileFormat] {
        &[
            FileFormat::Ini,
            FileFormat::Json,
            FileFormat::Json5,
            FileFormat::Toml,
            FileFormat::Yaml,
        ]
    }

    #[inline]
    pub fn file_format_len() -> usize {
        lazy_static! {
            static ref FILE_FORMAT_LEN: usize = FileBuilder::seq_file_format().len();
        }
        *FILE_FORMAT_LEN
    }

    pub fn with_input(sources: Vec<FileSource>) -> anyhow::Result<Self> {
        let mut builder = Config::builder();
        for s in sources.iter() {
            builder = builder.add_source(File::with_name(&s.abs).required(s.required));
        }

        Ok(Self { builder, sources })
    }

    pub fn with_preset() -> anyhow::Result<Self> {
        #[allow(unused_mut)]
        let mut sources = FileSource::preset_apps()?;

        let mut builder = Config::builder();
        let ffmt = FileBuilder::seq_file_format();
        for s in sources.iter() {
            for f in ffmt.iter() {
                builder = builder.add_source(File::new(&s.abs, *f).required(s.required));
            }
        }
        Ok(Self { builder, sources })
    }

    pub fn from_cmd_global_options(paths: Vec<PathBuf>) -> anyhow::Result<Self> {
        let mut sources = Vec::with_capacity(paths.len());
        for p in paths.into_iter() {
            sources.push(FileSource::new(
                "input_config",
                p.canonicalize()
                    .with_context(|| format!("invalid path `{}`", p.display()))?
                    .to_str()
                    .unwrap(),
                true,
                true,
            ));
        }
        Self::with_input(sources)
    }
}

#[derive(Default)]
pub struct FileDisplay {
    pub sources: Vec<FileSource>,
}

impl std::fmt::Debug for FileDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("_")
            .field(
                "format",
                &format_args!("{:?}", FileBuilder::seq_file_format()),
            )
            .field("sources", &self.sources)
            .finish()
    }
}

impl FileDisplay {
    pub fn new(sources: Vec<FileSource>) -> Self {
        let mut _self = Self { sources };
        FileBuilder::lazy_using(&mut _self.sources);
        _self
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::util;

    #[test]
    fn test_file_builder() -> anyhow::Result<()> {
        let mut configs = FileSource::preset_apps()?;
        configs.push(FileSource::new(
            "tests_dir_config",
            crate::workdir_join!("test.YamL").to_str().unwrap(),
            false,
            true,
        ));

        let mut fb = FileBuilder::with_input(configs)?;

        {
            let random_number = util::tests::random_number(2, 5);
            println!("test: repeat lazy_using {random_number}");
            for _ in 0..random_number {
                FileBuilder::lazy_using(&mut fb.sources);
            }
            crate::println_dash!(50);
        }

        let fd = FileDisplay::new(fb.sources);
        println!("{:#?}", fd);

        Ok(())
    }
}
