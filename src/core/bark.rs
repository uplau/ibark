use anyhow::{anyhow, Context};
use std::collections::HashMap;

/*
* Contributors should not print `Remote` and `Device` directly in release, see dump()
* Debug is not implemented just because of development
*/

pub struct Contexts;
impl Contexts {
    pub fn verify(i: HashMap<String, String>) -> anyhow::Result<HashMap<String, String>> {
        let mut update = HashMap::with_capacity(i.len());
        for (k, v) in i.into_iter() {
            if v.is_empty() {
                continue;
            }

            match k.to_lowercase().as_str() {
                "a" | "autocopy" | "automaticallycopy" => {
                    let v = "1"; // must be 1
                    update.insert("automaticallyCopy".into(), v.into()); // v2
                    update.insert("autoCopy".into(), v.into()); // v1
                }
                "bdg" | "badge" => match v.parse::<i32>() {
                    Ok(_) => {
                        update.insert("badge".into(), v);
                    }
                    Err(_) => return Err(anyhow!("bark_context_badge `{v}` not a number")),
                },
                "b" | "body" => {
                    // raw \n = \\n
                    update.insert("body".into(), v.replace("\\n", "\n"));
                }
                "c" | "copy" => {
                    // raw \n = \\n
                    update.insert("copy".into(), v.replace("\\n", "\n"));
                }
                "cat" | "category" => {
                    super::cli::Output::warn("bark_context_category is not used yet");
                    // update.insert("category".into(), v);
                }
                "g" | "group" => {
                    update.insert("group".into(), v);
                }
                "i" | "icon" => {
                    update.insert("icon".into(), v);
                }
                "isa" | "isarchive" => {
                    update.insert("isArchive".into(), "1".into());
                }
                "l" | "level" => {
                    let m = ["active", "timeSensitive", "passive"];
                    if m.contains(&v.as_str()) {
                        update.insert("level".into(), v);
                    } else {
                        return Err(anyhow!(
                            "bark_context_level `{v}` not match `{}`",
                            m.join("|")
                        ));
                    }
                }
                "s" | "sound" => {
                    // https://github.com/Finb/Bark/tree/master/Sounds
                    update.insert("sound".into(), v);
                }
                "t" | "title" => {
                    update.insert("title".into(), v.replace("\\n", "\n"));
                }
                "u" | "url" => {
                    update.insert("url".into(), v);
                }
                _ => return Err(anyhow!("unsupported bark_context `{k}`")),
            }
        }
        Ok(update)
    }
}

// maybe not Debug
#[derive(Debug, Default)]
pub struct Device {
    scheme: String,
    key: String,
    aes_mode: Option<String>,
    aes_key: Option<Vec<u8>>,
    aes_iv: Option<Vec<u8>>,
    aes_padding: Option<String>, // temporarily ignored
}

impl Device {
    pub fn find_merge(s: &HashMap<String, String>, f: Vec<String>) -> HashMap<String, String> {
        let mut merge = HashMap::with_capacity(f.len());
        for find in f.into_iter() {
            if find.is_empty() {
                continue;
            }
            if s.contains_key(&find) {
                if merge.contains_key(&find) {
                    continue;
                }
                let value = s.get(&find).unwrap().to_string();
                merge.insert(find, value);
            } else {
                let key = crate::util::hash_hex_string(&find);
                if merge.contains_key(&key) {
                    continue;
                }
                merge.insert(key, find);
            }
        }
        merge
    }

    pub fn new(input: &str) -> anyhow::Result<Self> {
        let u = url::Url::parse(input).with_context(|| Self::parse_context("input", input))?;

        Self::default()
            .parse_scheme(u.scheme())?
            .parse_key(u.host_str().unwrap_or_default())?
            .parse_aes(&u)
    }

    pub fn dump(input: &str) -> anyhow::Result<String> {
        let _self = Self::new(input)?;

        match _self.scheme.as_str() {
            "d" | "aes" => {
                let key_len = _self.key.len();
                let key_secret = "*".repeat(key_len);

                if _self.scheme == "d" {
                    Ok(format!("{}://{key_secret}", _self.scheme))
                } else {
                    let aes_key_len = _self.aes_key.unwrap().len();
                    let aes_secret = format!("{}:{}@", "*".repeat(aes_key_len), "*".repeat(16));

                    let path_segments = format!(
                        "{}/{}/{}",
                        aes_key_len * 8,
                        _self.aes_mode.unwrap(),
                        _self.aes_padding.unwrap()
                    );

                    Ok(format!(
                        "{}://{aes_secret}{key_secret}/{path_segments}",
                        _self.scheme
                    ))
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn new_request(
        input: &str,
        client: &reqwest::Client,
        remote: &str,
        contexts: &HashMap<String, String>,
    ) -> anyhow::Result<reqwest::RequestBuilder> {
        let _self = Self::new(input)?;
        let mut request = client.post(format!("{remote}/{}", _self.key));

        match _self.scheme.as_str() {
            "d" => {
                request = request.form(contexts);
            }
            "aes" => {
                let data = serde_json::to_vec(contexts)?;

                let cipher = match _self.aes_mode.unwrap().as_str() {
                    "cbc" => {
                        let bitlen = _self.aes_key.as_deref().unwrap().len() * 8;
                        match bitlen {
                            128 => openssl::symm::Cipher::aes_128_cbc(),
                            192 => openssl::symm::Cipher::aes_192_cbc(),
                            256 => openssl::symm::Cipher::aes_256_cbc(),
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    "ecb" => {
                        let bitlen = _self.aes_key.as_deref().unwrap().len() * 8;
                        match bitlen {
                            128 => openssl::symm::Cipher::aes_128_ecb(),
                            192 => openssl::symm::Cipher::aes_192_ecb(),
                            256 => openssl::symm::Cipher::aes_256_ecb(),
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    _ => unreachable!(),
                };

                let ciphertext_base64 = openssl::base64::encode_block(&openssl::symm::encrypt(
                    cipher,
                    &_self.aes_key.unwrap(),
                    _self.aes_iv.as_deref(),
                    &data,
                )?);

                let params = HashMap::from([("ciphertext", ciphertext_base64.as_str())]);
                request = request.form(&params);
            }
            _ => unreachable!(),
        }

        Ok(request)
    }

    fn parse_context(t: &str, input: &str) -> String {
        format!("parse bark_device_{t} `{input}` failed")
    }

    fn parse_scheme(mut self, scheme: &str) -> anyhow::Result<Self> {
        match scheme {
            "d" | "aes" => self.scheme = scheme.to_string(),
            _ => return Err(anyhow!("unsupported bark_device_scheme `{scheme}`")),
        }
        Ok(self)
    }

    fn parse_key(mut self, key: &str) -> anyhow::Result<Self> {
        let len = key.len();
        let len_check: usize = 22;
        if len < len_check {
            let secret = "*".repeat(len);
            return Err(anyhow!("len `{len}` < {len_check}"))
                .with_context(|| Self::parse_context("key", &secret));
        }
        self.key = key.to_string();
        Ok(self)
    }

    fn parse_aes(mut self, u: &url::Url) -> anyhow::Result<Self> {
        match self.scheme.as_str() {
            "aes" => {
                let path_segments: Vec<_> = u
                    .path_segments()
                    .with_context(|| "path segments is none")
                    .with_context(|| {
                        Self::parse_context("aes", "aes://aes_key:aes_iv@device_key/^here")
                    })?
                    .collect();

                let count = path_segments.len();
                if count < 3 {
                    return Err(anyhow!("count `{count}` < 3"))
                        .with_context(|| Self::parse_context("aes_path_segments", u.path()));
                }

                // let aes_key_bitlen = *path_segments.get(0).unwrap();
                let aes_key_bitlen = *path_segments.first().unwrap();
                match aes_key_bitlen {
                    "128" | "192" | "256" => {
                        let aes_key_bitlen = aes_key_bitlen.parse::<usize>()?;
                        let input_aes_key_bitlen = u.username().len() * 8;

                        if input_aes_key_bitlen != aes_key_bitlen {
                            return Err(anyhow!(
                                "bitlen `{input_aes_key_bitlen}` not match `{aes_key_bitlen}`"
                            ))
                            .with_context(|| {
                                Self::parse_context(
                                    "aes_key",
                                    &format!("aes://^here:aes_iv@device_key/{aes_key_bitlen}"),
                                )
                            });
                        }

                        let aes_iv = u.password().unwrap_or_default();
                        let input_aes_iv_bitlen = aes_iv.len() * 8;
                        if input_aes_iv_bitlen != 128 {
                            return Err(anyhow!("bitlen `{input_aes_iv_bitlen}` not match `128`"))
                                .with_context(|| {
                                    Self::parse_context("aes_iv", "aes://aes_key:^here")
                                });
                        }

                        self.aes_key = Some(hex::decode(hex::encode(u.username()))?);
                        self.aes_iv = Some(hex::decode(hex::encode(aes_iv))?);
                    }
                    _ => {
                        return Err(anyhow!(
                            "unsupported bark_device_aes_key_bitlen `{aes_key_bitlen}`"
                        ));
                    }
                }

                let aes_mode = *path_segments.get(1).unwrap();
                match aes_mode {
                    "cbc" | "ecb" => self.aes_mode = Some(aes_mode.to_string()),
                    _ => {
                        return Err(anyhow!("unsupported bark_device_aes_mode `{aes_mode}`"));
                    }
                }

                let aes_padding = *path_segments.get(2).unwrap();
                match aes_padding {
                    "pkcs7" => self.aes_padding = Some(aes_padding.to_string()),
                    _ => {
                        return Err(anyhow!(
                            "unsupported bark_device_aes_padding `{aes_padding}`"
                        ));
                    }
                }

                Ok(self)
            }
            _ => Ok(self),
        }
    }
}

pub struct Remote;
impl Remote {
    pub fn verify(remote: &str) -> anyhow::Result<()> {
        let url = url::Url::parse(remote)
            .with_context(|| format!("parse bark_remote `{remote}` failed"))?;

        let scheme = url.scheme();
        if !["http", "https"].contains(&scheme) {
            return Err(anyhow!("unsupported bark_remote_scheme `{scheme}`"));
        }
        Ok(())
    }

    pub fn dump(remote: &str) -> anyhow::Result<String> {
        let mut url = url::Url::parse(remote)?;
        if url.password().is_some() {
            let len = url.password().unwrap().len();
            url.set_password(Some(&"*".repeat(len))).unwrap();
        }
        Ok(url.into())
    }

    pub fn scheme_host_port(remote: &str) -> anyhow::Result<String> {
        let url = url::Url::parse(remote)?;
        let host = url.host_str().unwrap_or("unknown_host");
        let port = url.port_or_known_default().unwrap_or(0);
        Ok(format!("{}://{host}:{port}", url.scheme()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::tests::*;

    #[test]
    fn test_contexts() -> anyhow::Result<()> {
        dbg!(Contexts::verify(
            crate::hash_map! {
                "a"=>"true",
                "t"=>"iBark 💗",
                "b"=>r#"Hello 👋\nWorld🌍"#,
                "bdg"=>"100",
                "l"=>"active",
                "cat"=>"test",
                "g"=>""
            }
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        )?);
        Ok(())
    }

    #[test]
    fn test_device() -> anyhow::Result<()> {
        let gen_args = |is_err: bool| {
            let mut args = Vec::with_capacity(5);

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
        };

        let args = gen_args(false);
        dbg!(&args);
        crate::println_dash!(50);

        for input in args.clone().into_iter() {
            dbg!(Device::dump(&input)?);
            crate::println_dash!(20);
        }

        for input in args.into_iter() {
            dbg!(Device::new_request(
                input.as_ref(),
                &reqwest::Client::new(),
                &format!("https://{}.com", random_string(10)),
                &HashMap::new()
            )?);
            crate::println_dash!(20);
        }

        Ok(())
    }

    #[test]
    fn test_remote() -> anyhow::Result<()> {
        dbg!(Remote::verify("https://hello.world:65535")?);
        dbg!(Remote::dump("https://username:password@hello.world:65534")?);
        Ok(())
    }
}
