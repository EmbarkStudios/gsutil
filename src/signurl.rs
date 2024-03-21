use crate::util;
use anyhow::Context as _;
use std::time::Duration;
use tame_gcs::{http, signed_url, signing};

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
    Resumable,
}

#[derive(Clone)]
struct Dur(Duration);

impl std::str::FromStr for Dur {
    type Err = clap::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let suffix_pos = src.find(char::is_alphabetic).unwrap_or(src.len());

        let num: u64 = src[..suffix_pos]
            .parse()
            .map_err(|err| clap::Error::raw(clap::error::ErrorKind::ValueValidation, err))?;
        let suffix = if suffix_pos == src.len() {
            "h"
        } else {
            &src[suffix_pos..]
        };

        let duration = match suffix {
            "s" | "S" => Duration::from_secs(num),
            "m" | "M" => Duration::from_secs(num * 60),
            "h" | "H" => Duration::from_secs(num * 60 * 60),
            "d" | "D" => Duration::from_secs(num * 60 * 60 * 24),
            s => {
                return Err(clap::Error::raw(
                    clap::error::ErrorKind::ValueValidation,
                    format!("unknown duration suffix '{s}'"),
                ))
            }
        };

        Ok(Self(duration))
    }
}

#[derive(clap::Parser)]
pub struct Args {
    /// The HTTP method to be used with the signed url.
    #[clap(short, default_value = "GET", ignore_case = true)]
    method: Method,
    #[clap(
        short,
        default_value = "1h",
        long_help = "The duration that the signed url will be valid for.

Times may be specified with no suffix (default hours), or one of:
* (s)econds
* (m)inutes
* (h)ours
* (d)ays

"
    )]
    duration: Dur,
    /// The content-type for which the url is valid for, eg. "application/json"
    #[structopt(short)]
    content_type: Option<String>,
    /// The gs:// url
    url: url::Url,
}

pub async fn cmd(cred_path: std::path::PathBuf, args: Args) -> anyhow::Result<()> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let url_signer = signed_url::UrlSigner::with_ring();
    let service_account = signing::ServiceAccount::load_json_file(cred_path)?;

    let mut options = signed_url::SignedUrlOptional {
        duration: args.duration.0,
        ..Default::default()
    };

    if let Some(content_type) = args.content_type {
        options.headers.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_str(&content_type)?,
        );
    }

    options.method = match args.method {
        Method::Get => http::Method::GET,
        Method::Post => http::Method::POST,
        Method::Put => http::Method::PUT,
        Method::Delete => http::Method::DELETE,
        Method::Head => http::Method::HEAD,
        Method::Options => http::Method::OPTIONS,
        Method::Connect => http::Method::CONNECT,
        Method::Patch => http::Method::PATCH,
        Method::Trace => http::Method::TRACE,
        Method::Resumable => {
            options.headers.insert(
                http::header::HeaderName::from_static("x-goog-resumable"),
                http::header::HeaderValue::from_static("start"),
            );
            http::Method::POST
        }
    };

    let signed_url = url_signer.generate(
        &service_account,
        &(
            oid.bucket(),
            oid.object().context("must have a valid object name")?,
        ),
        options,
    )?;

    println!("{}", signed_url);

    Ok(())
}
