use crate::util;
use anyhow::Context as _;
use std::{convert::TryFrom, path::PathBuf};
use tame_gcs::objects::{self, Metadata};

#[derive(clap::ValueEnum, Clone, Copy)]
enum Acl {
    ProjectPrivate,
    Private,
    PublicRead,
    AuthenticatedRead,
    BucketOwnerRead,
    BucketOwnerFullControl,
}

impl From<Acl> for tame_gcs::common::PredefinedAcl {
    fn from(a: Acl) -> Self {
        match a {
            Acl::ProjectPrivate => Self::ProjectPrivate,
            Acl::Private => Self::Private,
            Acl::PublicRead => Self::PublicRead,
            Acl::AuthenticatedRead => Self::AuthenticatedRead,
            Acl::BucketOwnerRead => Self::BucketOwnerRead,
            Acl::BucketOwnerFullControl => Self::BucketOwnerFullControl,
        }
    }
}

#[derive(clap::Parser)]
pub struct Args {
    /// Predefined ACL to apply to the destination GCS object
    #[clap(short = 'a')]
    predef_acl: Option<Acl>,
    /// A gs: URL or filepath for the source path to copy from,
    /// wildcards are not currently supported
    src_url: String,
    /// A gs: URL or filepath for the destination to copy to,
    /// wildcards are not currently supported
    dest_url: String,
}

enum DataPath {
    Gs(util::GsUrl),
    Local(PathBuf),
}

impl DataPath {
    #[inline]
    fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }
}

impl TryFrom<String> for DataPath {
    type Error = anyhow::Error;

    fn try_from(s: String) -> anyhow::Result<Self> {
        if s.starts_with("gs://") {
            let url = url::Url::parse(&s)?;
            Ok(Self::Gs(util::gs_url_to_object_id(&url)?))
        } else {
            Ok(Self::Local(PathBuf::from(s)))
        }
    }
}

// cp is probably gsutil's most complicated subcommand, so we only implement
// a bare minimum
pub async fn cmd(ctx: &util::RequestContext, args: Args) -> anyhow::Result<()> {
    use std::fs;

    let src = DataPath::try_from(args.src_url)?;
    let dst = DataPath::try_from(args.dest_url)?;

    // Just support gcs to local or vice versa, not local to local or gcs to gcs
    if src.is_local() == dst.is_local() {
        let location = if src.is_local() { "local disk" } else { "gcs" };

        anyhow::bail!("source and destination are both located on {location}")
    }

    match (&src, &dst) {
        (DataPath::Local(ref src), DataPath::Gs(dst)) => {
            let src_file = fs::File::open(src).context("source path")?;
            let src_len = src_file.metadata()?.len();

            let optional = args.predef_acl.map(|acl| objects::InsertObjectOptional {
                predefined_acl: Some(acl.into()),
                ..Default::default()
            });

            let insert_req = ctx.obj.insert_multipart(
                dst.bucket(),
                src_file,
                src_len,
                &Metadata {
                    name: dst.object().map(|obn| obn.to_string()),
                    content_encoding: Some("identity".to_owned()),
                    ..Default::default()
                },
                optional,
            )?;

            let _insert_res: objects::InsertResponse = util::execute(ctx, insert_req).await?;

            Ok(())
        }
        (DataPath::Gs(src), DataPath::Local(dst)) => {
            let mut dst_file = fs::File::create(dst).context("destination path")?;

            let dl_req = ctx.obj.download(
                &(
                    src.bucket(),
                    src.object()
                        .context("must provide a full object name to copy from")?,
                ),
                None,
            )?;

            let mut response: objects::DownloadObjectResponse = util::execute(ctx, dl_req).await?;

            std::io::copy(&mut response, &mut dst_file)?;

            Ok(())
        }
        _ => unreachable!(),
    }
}
