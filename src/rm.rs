use crate::util;
use anyhow::Context as _;

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// The gs:// url to the object
    url: url::Url,
}

pub async fn cmd(ctx: &util::RequestContext, args: Args) -> anyhow::Result<()> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let del_req = ctx.obj.delete(
        &(
            oid.bucket(),
            oid.object().context("invalid object name specified")?,
        ),
        None,
    )?;

    util::execute::<_, tame_gcs::objects::DeleteObjectResponse>(ctx, del_req).await?;

    Ok(())
}
