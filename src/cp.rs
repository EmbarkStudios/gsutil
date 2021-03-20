use crate::util;
use anyhow::{anyhow, bail, Context, Error};
use std::{convert::TryFrom, path::PathBuf};
use structopt::StructOpt;
use tame_gcs::objects::{self, Metadata, Object};

#[derive(StructOpt, Debug)]
pub struct Args {
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
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
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
pub async fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    use std::fs;

    let src = DataPath::try_from(args.src_url)?;
    let dst = DataPath::try_from(args.dest_url)?;

    // Just support gcs to local or vice versa, not local to local or gcs to gcs
    if src.is_local() == dst.is_local() {
        let location = if src.is_local() { "local disk" } else { "gcs" };

        bail!("source and destination are both located on {}", location)
    }

    match (&src, &dst) {
        (DataPath::Local(ref src), DataPath::Gs(dst)) => {
            let src_file = fs::File::open(src).context("source path")?;
            let src_len = src_file.metadata()?.len();

            let obj_name = format!(
                "{}{}{}",
                dst.object().map_or("", |on| on.as_ref()),
                if dst.object().is_some() { "/" } else { "" },
                src.file_name()
                    .as_ref()
                    .and_then(|os| os.to_str())
                    .ok_or_else(|| anyhow!("can't turn file_name into string"))?
            );
            let insert_req = Object::insert_multipart(
                dst.bucket(),
                src_file,
                src_len,
                &Metadata {
                    name: Some(obj_name),
                    content_encoding: Some("identity".to_owned()),
                    ..Default::default()
                },
                None,
            )?;

            let _insert_res: objects::InsertResponse = util::execute(ctx, insert_req).await?;

            Ok(())
        }
        (DataPath::Gs(src), DataPath::Local(dst)) => {
            let mut dst_file = fs::File::create(dst).context("destination path")?;

            let dl_req = Object::download(
                &(
                    src.bucket(),
                    src.object()
                        .ok_or_else(|| anyhow!("must provide a full object name to copy from"))?,
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
