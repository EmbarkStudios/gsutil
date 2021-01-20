use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    /// Concatenate object content to stdout
    #[structopt(name = "cat")]
    Cat(gsutil::cat::Args),
    /// Copy files and objects
    #[structopt(name = "cp")]
    Cp(gsutil::cp::Args),
    /// List objects
    #[structopt(name = "ls")]
    Ls(gsutil::ls::Args),
    /// Remove objects
    #[structopt(name = "rm")]
    Rm(gsutil::rm::Args),
    /// Set metadata on objects
    #[structopt(name = "setmeta")]
    SetMeta(gsutil::setmeta::Args),
    /// Create a signed url
    #[structopt(name = "signurl")]
    Signurl(gsutil::signurl::Args),
    /// Display object status
    #[structopt(name = "stat")]
    Stat(gsutil::stat::Args),
}

#[derive(StructOpt)]
#[structopt(name = "gsutil")]
struct Opts {
    /// Path to a service account credentials file used to obtain oauth2 tokens.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        env = "GOOGLE_APPLICATION_CREDENTIALS"
    )]
    credentials: Option<std::path::PathBuf>,
    #[structopt(subcommand)]
    cmd: Command,
}

async fn real_main() -> Result<(), anyhow::Error> {
    let args = Opts::from_args();

    let cred_path = args
        .credentials
        .ok_or_else(|| anyhow::anyhow!("credentials not specified"))?;

    let client = reqwest::Client::builder().build()?;
    let svc_account_info =
        tame_oauth::gcp::ServiceAccountInfo::deserialize(std::fs::read_to_string(&cred_path)?)?;
    let svc_account_access = tame_oauth::gcp::ServiceAccountAccess::new(svc_account_info)?;

    let ctx = gsutil::util::RequestContext {
        client,
        cred_path,
        auth: std::sync::Arc::new(svc_account_access),
    };

    match args.cmd {
        Command::Cat(args) => gsutil::cat::cmd(&ctx, args).await,
        Command::Cp(args) => gsutil::cp::cmd(&ctx, args).await,
        Command::Ls(args) => gsutil::ls::cmd(&ctx, args).await,
        Command::Rm(args) => gsutil::rm::cmd(&ctx, args).await,
        Command::SetMeta(args) => gsutil::setmeta::cmd(&ctx, args).await,
        Command::Signurl(args) => gsutil::signurl::cmd(&ctx, args).await,
        Command::Stat(args) => gsutil::stat::cmd(&ctx, args).await,
    }
}

#[tokio::main]
async fn main() {
    match real_main().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", ansi_term::Color::Red.paint(format!("{:?}", e)));
            std::process::exit(1);
        }
    }
}
