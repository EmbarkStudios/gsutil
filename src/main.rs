// BEGIN - Embark standard lints v5 for Rust 1.55+
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.5 for Rust 1.55+
// crate-specific exceptions:

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

async fn real_main() -> Result<(), anyhow::Error> {
    use anyhow::Context as _;
    use clap::Parser;

    let args = Opts::parse();

    let client = reqwest::Client::builder().build()?;

    let token_provider = match &args.credentials {
        Some(cred_path) => {
            let svc_account_info = tame_oauth::gcp::ServiceAccountInfo::deserialize(
                std::fs::read_to_string(cred_path)?,
            )?;
            tame_oauth::gcp::TokenProviderWrapper::ServiceAccount(
                tame_oauth::gcp::ServiceAccountProvider::new(svc_account_info)?,
            )
        }
        None => tame_oauth::gcp::TokenProviderWrapper::get_default_provider()?
            .context("unable to determine default token provider")?,
    };

    let ctx = gsutil::util::RequestContext {
        client,
        auth: std::sync::Arc::new(token_provider),
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
            eprintln!("{}", ansi_term::Color::Red.paint(format!("{:?}", e)));
            #[allow(clippy::exit)]
            std::process::exit(1);
        }
    }
}
