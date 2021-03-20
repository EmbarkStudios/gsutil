// BEGIN - Embark standard lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.3

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
            #[allow(clippy::exit)]
            std::process::exit(1);
        }
    }
}
