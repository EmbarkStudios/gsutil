use anyhow::Context as _;
use std::sync::Arc;
use tame_gcs::{self as tgcs, http};
use tame_oauth::gcp as oauth;

/// Converts a vanilla `http::Request` into a `reqwest::Request`
async fn convert_request<B>(
    req: http::Request<B>,
    client: &reqwest::Client,
) -> anyhow::Result<reqwest::Request>
where
    B: std::io::Read + Send + 'static,
{
    let (parts, mut body) = req.into_parts();

    let uri = parts.uri.to_string();

    let builder = match parts.method {
        http::Method::GET => client.get(&uri),
        http::Method::POST => client.post(&uri),
        http::Method::DELETE => client.delete(&uri),
        http::Method::PATCH => client.patch(&uri),
        http::Method::PUT => client.put(&uri),
        method => anyhow::bail!("'{method}' not implemented"),
    };

    let content_len = tame_gcs::util::get_content_length(&parts.headers).unwrap_or(0);
    let mut buffer = bytes::BytesMut::with_capacity(content_len);

    let mut block = [0u8; 8 * 1024];

    loop {
        let read = body.read(&mut block)?;

        if read > 0 {
            buffer.extend_from_slice(&block[..read]);
        } else {
            break;
        }
    }

    Ok(builder
        .headers(parts.headers)
        .body(buffer.freeze())
        .build()?)
}

/// Converts a `reqwest::Response` into a vanilla `http::Response`. This currently copies
/// the entire response body into a single buffer with no streaming
async fn convert_response(res: reqwest::Response) -> anyhow::Result<http::Response<bytes::Bytes>> {
    let mut builder = http::Response::builder()
        .status(res.status())
        .version(res.version());

    let headers = builder
        .headers_mut()
        .context("failed to convert response headers")?;

    headers.extend(
        res.headers()
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    let content_len = tame_gcs::util::get_content_length(headers).unwrap_or_default();
    let mut buffer = bytes::BytesMut::with_capacity(content_len);

    let mut stream = res.bytes_stream();

    use bytes::BufMut;
    use futures_util::StreamExt;

    while let Some(item) = stream.next().await {
        buffer.put(item?);
    }

    Ok(builder.body(buffer.freeze())?)
}

#[derive(Clone)]
pub struct RequestContext {
    pub client: reqwest::Client,
    pub auth: Arc<oauth::TokenProviderWrapper>,
    pub obj: tgcs::objects::Object,
}

/// Executes a GCS request via a reqwest client and returns the parsed response/API error
pub async fn execute<B, R>(ctx: &RequestContext, mut req: http::Request<B>) -> anyhow::Result<R>
where
    R: tgcs::ApiResponse<bytes::Bytes>,
    B: std::io::Read + Send + 'static,
{
    use oauth::TokenProvider;

    // First, get our oauth token, which can mean we have to do an additional
    // request if we've never retrieved one yet, or the one we are using has expired
    let token = match ctx.auth.get_token(&[tame_gcs::Scopes::FullControl])? {
        oauth::TokenOrRequest::Token(token) => token,
        oauth::TokenOrRequest::Request {
            request,
            scope_hash,
            ..
        } => {
            let (parts, body) = request.into_parts();
            let read_body = std::io::Cursor::new(body);
            let new_request = http::Request::from_parts(parts, read_body);

            let req = convert_request(new_request, &ctx.client)
                .await
                .context("failed to create token request")?;
            let res = ctx
                .client
                .execute(req)
                .await
                .context("failed to send token request")?;

            let response = convert_response(res)
                .await
                .context("failed to convert token response")?;

            ctx.auth
                .parse_token_response(scope_hash, response)
                .context("failed to parse token response")?
        }
    };

    // Add the authorization token, note that the tame-oauth crate will automatically
    // set the HeaderValue correctly, in the GCP case this is usually "Bearer <token>"
    req.headers_mut()
        .insert(http::header::AUTHORIZATION, token.try_into()?);

    let request = convert_request(req, &ctx.client).await?;
    let response = ctx.client.execute(request).await?;
    let response = convert_response(response)
        .await
        .context("failed to convert response")?;

    Ok(R::try_from_parts(response)?)
}

pub struct GsUrl {
    bucket_name: tgcs::BucketName<'static>,
    obj_name: Option<tgcs::ObjectName<'static>>,
}

impl GsUrl {
    pub fn bucket(&self) -> &tame_gcs::BucketName<'_> {
        &self.bucket_name
    }

    pub fn object(&self) -> Option<&tame_gcs::ObjectName<'_>> {
        self.obj_name.as_ref()
    }
}

/// Converts a `gs://<bucket_name>/<object_name>` url into a regular object identifier
pub fn gs_url_to_object_id(url: &url::Url) -> anyhow::Result<GsUrl> {
    match url.scheme() {
        "gs" => {
            let bucket_name = url.host_str().context("no bucket specified")?;
            // Skip first /
            let object_name = if url.path().is_empty() {
                ""
            } else {
                &url.path()[1..]
            };

            Ok(GsUrl {
                bucket_name: tame_gcs::BucketName::try_from(String::from(bucket_name))?,
                obj_name: tame_gcs::ObjectName::try_from(String::from(object_name)).ok(),
            })
        }
        scheme => anyhow::bail!("invalid url scheme: {scheme}"),
    }
}
