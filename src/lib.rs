#![forbid(unsafe_code)]
use axum::extract::RequestParts;
use hyper::http::header;
use rejection::{BodyAlreadyExtracted, HeadersAlreadyExtracted};

mod error;
mod util;
mod rejection;

#[cfg(test)]
mod test_helpers;

mod msgpack;
mod msgpack_raw;

pub use msgpack::MsgPack;
pub use msgpack_raw::MsgPackRaw;

pub(crate) fn has_content_type<B>(
    req: &RequestParts<B>,
    expected_content_type: &str,
) -> Result<bool, HeadersAlreadyExtracted> {
    let content_type = if let Some(content_type) = req
        .headers()
        .ok_or(HeadersAlreadyExtracted)?
        .get(header::CONTENT_TYPE)
    {
        content_type
    } else {
        return Ok(false);
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return Ok(false);
    };

    Ok(content_type.starts_with(expected_content_type))
}

pub(crate) fn take_body<B>(req: &mut RequestParts<B>) -> Result<B, BodyAlreadyExtracted> {
    req.take_body().ok_or(BodyAlreadyExtracted)
}

