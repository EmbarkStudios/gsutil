use crate::util;
use anyhow::Error;
use tame_gcs::objects::Object;

#[derive(clap::Parser, Debug)]
pub struct Args {
    /// The gs:// url to the object to stat
    url: url::Url,
}

pub async fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let get_req = Object::get(
        &(
            oid.bucket(),
            oid.object()
                .ok_or_else(|| anyhow::anyhow!("invalid object name specified"))?,
        ),
        None,
    )?;
    let get_res: tame_gcs::objects::GetObjectResponse = util::execute(ctx, get_req).await?;

    let md = get_res.metadata;

    // Print out the information the same way gsutil does, except with RFC-2822 date formatting
    println!("{}", ansi_term::Color::Cyan.paint(args.url.as_str()));
    println!(
        "    Creation time:\t{}",
        md.time_created
            .expect("time_created")
            .format(&time::format_description::well_known::Rfc2822)
            .unwrap()
    );
    println!(
        "    Update time:\t{}",
        md.updated
            .expect("updated")
            .format(&time::format_description::well_known::Rfc2822)
            .unwrap()
    );
    println!(
        "    Storage class:\t{}",
        md.storage_class.expect("storage_class")
    );
    println!("    Content-Length:\t{}", md.size.expect("size"));
    println!(
        "    Content-Type:\t{}",
        md.content_type.as_deref().unwrap_or("None")
    );

    if let Some(md) = &md.metadata {
        for (k, v) in md {
            println!("        {}:\t\t{}", k, v);
        }
    }

    println!("    Hash (crc32c):\t{}", md.crc32c.expect("crc32c"));
    println!("    Hash (md5):\t\t{}", md.md5_hash.expect("md5_hash"));
    println!("    ETag:\t\t{}", md.etag.expect("etag"));
    println!("    Generation:\t\t{}", md.generation.expect("generation"));
    println!(
        "    Metageneration:\t{}",
        md.metageneration.expect("metageneration")
    );

    Ok(())
}
