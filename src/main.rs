#[derive(clap::Subcommand)]
enum Command {
    /// Concatenate object content to stdout
    #[clap(name = "cat")]
    Cat(gsutil::cat::Args),
    /// Copy files and objects
    #[clap(name = "cp")]
    Cp(gsutil::cp::Args),
    /// List objects
    #[clap(name = "ls")]
    Ls(gsutil::ls::Args),
    /// Remove objects
    #[clap(name = "rm")]
    Rm(gsutil::rm::Args),
    /// Set metadata on objects
    #[clap(name = "setmeta")]
    SetMeta(gsutil::setmeta::Args),
    /// Create a signed url
    #[clap(name = "signurl")]
    Signurl(gsutil::signurl::Args),
    /// Display object status
    #[clap(name = "stat")]
    Stat(gsutil::stat::Args),
}

#[derive(clap::Parser)]
#[clap(name = "gsutil")]
struct Opts {
    /// Path to a service account credentials file used to obtain oauth2 tokens.
    #[clap(short, long, env = "GOOGLE_APPLICATION_CREDENTIALS")]
    credentials: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    cmd: Command,
}

async fn real_main() -> anyhow::Result<()> {
    use anyhow::Context as _;
    use clap::Parser;

    let args = Opts::parse();

    let client = reqwest::Client::builder().build()?;

    let token_provider = match &args.credentials {
        Some(cred_path) => {
            let svc_account_info = tame_oauth::gcp::ServiceAccountInfo::deserialize(
                std::fs::read_to_string(cred_path)?,
            )?;
            tame_oauth::gcp::TokenProviderWrapper::wrap(
                tame_oauth::gcp::TokenProviderWrapperInner::ServiceAccount(
                    tame_oauth::gcp::service_account::ServiceAccountProviderInner::new(
                        svc_account_info,
                    )?,
                ),
            )
        }
        None => tame_oauth::gcp::TokenProviderWrapper::get_default_provider()?
            .context("unable to determine default token provider")?,
    };

    let ctx = gsutil::util::RequestContext {
        client,
        auth: std::sync::Arc::new(token_provider),
        obj: tame_gcs::objects::Object::default(),
    };

    match args.cmd {
        Command::Cat(args) => gsutil::cat::cmd(&ctx, args).await,
        Command::Cp(args) => gsutil::cp::cmd(&ctx, args).await,
        Command::Ls(args) => gsutil::ls::cmd(&ctx, args).await,
        Command::Rm(args) => gsutil::rm::cmd(&ctx, args).await,
        Command::SetMeta(args) => gsutil::setmeta::cmd(&ctx, args).await,
        Command::Signurl(sargs) => {
            use anyhow::Context as _;
            let cred_path = args
                .credentials
                .context("credentials required for URL signing")?;

            gsutil::signurl::cmd(cred_path, sargs).await
        }
        Command::Stat(args) => gsutil::stat::cmd(&ctx, args).await,
    }
}

#[tokio::main]
async fn main() {
    match real_main().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", nu_ansi_term::Color::Red.paint(format!("{e:?}")));
            #[allow(clippy::exit)]
            std::process::exit(1);
        }
    }
}
