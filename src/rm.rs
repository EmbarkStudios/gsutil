use crate::util;
use tame_gcs::objects::Object;

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// The gs:// url to the object
    url: url::Url,
}

pub async fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), anyhow::Error> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let del_req = Object::delete(
        &(
            oid.bucket(),
            oid.object()
                .ok_or_else(|| anyhow::anyhow!("invalid object name specified"))?,
        ),
        None,
    )?;

    util::execute::<_, tame_gcs::objects::DeleteObjectResponse>(ctx, del_req).await?;

    Ok(())
}
