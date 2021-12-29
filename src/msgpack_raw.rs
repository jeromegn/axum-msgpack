use std::{
    ops::{Deref, DerefMut},
};

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
    BoxError, body::BoxBody,
};
use axum::{
    body,
    http::{header::HeaderValue, StatusCode},
};

use hyper::{body::Buf, header, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    rejection::{InvalidMsgPackBody, MissingMsgPackContentType, MsgPackRejection},
    util::has_content_type,
    util::take_body,
};

/// MsgPack with no named fields
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPackRaw<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for MsgPackRaw<T>
where
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = MsgPackRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if has_content_type(req, "application/msgpack")? {
            let body = take_body(req)?;

            let buf = hyper::body::aggregate(body)
                .await
                .map_err(InvalidMsgPackBody::from_err)?;

            let value =
                rmp_serde::decode::from_read(buf.reader()).map_err(InvalidMsgPackBody::from_err)?;
            Ok(MsgPackRaw(value))
        } else {
            Err(MissingMsgPackContentType.into())
        }
    }
}


impl<T> IntoResponse for MsgPackRaw<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response<BoxBody> {
        let bytes = match rmp_serde::encode::to_vec(&self.0) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref())
                    )
                    .body(body::boxed(body::Full::from(err.to_string())))
                    .unwrap();
            }
        };

        let mut res = Response::new(body::boxed(body::Full::from(bytes)));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_MSGPACK.as_ref()),
        );
        res
    }
}


impl<T> Deref for MsgPackRaw<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MsgPackRaw<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for MsgPackRaw<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use axum::{routing::post, Json, Router};
    use serde::Deserialize;

    #[tokio::test]
    async fn deserialize_body() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }
        let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        let client = TestClient::new(app);

        // let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        // let client = TestClient::new(app);
        // let res = client.post("/").json(&json!({ "foo": "bar" })).send().await;
        // let body = res.text().await;

        // assert_eq!(body, "bar");
    }
}
