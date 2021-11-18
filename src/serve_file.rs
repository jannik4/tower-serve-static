use super::{AsyncReadBody, DEFAULT_CAPACITY};
use bytes::Bytes;
use http::{header, HeaderValue, Response};
use http_body::{combinators::BoxBody, Body};
use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tower_service::Service;

/// A file.
#[derive(Clone, Debug)]
pub struct File {
    bytes: &'static [u8],
    mime: HeaderValue,
}

impl File {
    /// Create a new [`File`].
    pub fn new(bytes: &'static [u8], mime: HeaderValue) -> Self {
        File { bytes, mime }
    }
}

/// Create a new [`File`].
///
/// The `Content-Type` will be guessed from the file extension.
#[macro_export]
macro_rules! include_file {
    ($file:expr) => {
        $crate::File::new(
            ::std::include_bytes!(::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), $file)),
            $crate::private::mime_guess::from_path(&$file)
                .first_raw()
                .map(|mime| $crate::private::http::HeaderValue::from_static(mime))
                .unwrap_or_else(|| {
                    $crate::private::http::HeaderValue::from_str(
                        $crate::private::mime::APPLICATION_OCTET_STREAM.as_ref(),
                    )
                    .unwrap()
                }),
        )
    };
}

/// Create a new [`File`] with a specific mime type.
///
/// # Panics
///
/// Will panic if the mime type isn't a valid [header value].
///
/// [header value]: https://docs.rs/http/latest/http/header/struct.HeaderValue.html
#[macro_export]
macro_rules! include_file_with_mime {
    ($file:expr, $mime:expr) => {
        $crate::File {
            bytes: ::std::include_bytes!(::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), $file)),
            mime: $crate::private::http::HeaderValue::from_str($mime.as_ref())
                .expect("mime isn't a valid header value"),
        }
    };
}

/// Service that serves a file.
#[derive(Clone, Debug)]
pub struct ServeFile {
    file: File,
    buf_chunk_size: usize,
}

impl ServeFile {
    /// Create a new [`ServeFile`].
    pub fn new(file: File) -> Self {
        Self {
            file,
            buf_chunk_size: DEFAULT_CAPACITY,
        }
    }

    /// Set a specific read buffer chunk size.
    ///
    /// The default capacity is 64kb.
    pub fn with_buf_chunk_size(mut self, chunk_size: usize) -> Self {
        self.buf_chunk_size = chunk_size;
        self
    }
}

impl<R> Service<R> for ServeFile {
    type Response = Response<ResponseBody>;
    type Error = io::Error;
    type Future = ResponseFuture;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: R) -> Self::Future {
        ResponseFuture {
            file: Some(self.file.clone()),
            buf_chunk_size: self.buf_chunk_size,
        }
    }
}

/// Response future of [`ServeFile`].
pub struct ResponseFuture {
    file: Option<File>,
    buf_chunk_size: usize,
}

impl Future for ResponseFuture {
    type Output = io::Result<Response<ResponseBody>>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let file = self.file.take().unwrap();

        let chunk_size = self.buf_chunk_size;
        let body = AsyncReadBody::with_capacity(file.bytes, chunk_size).boxed();
        let body = ResponseBody(body);

        let mut res = Response::new(body);
        res.headers_mut().insert(header::CONTENT_TYPE, file.mime);

        Poll::Ready(Ok(res))
    }
}

opaque_body! {
    /// Response body for [`ServeFile`].
    pub type ResponseBody = BoxBody<Bytes, io::Error>;
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use http::Request;
    use http_body::Body as _;
    use hyper::Body;
    use tower::ServiceExt;

    #[tokio::test]
    async fn basic() {
        let svc = ServeFile::new(include_file!("/README.md"));

        let res = svc.oneshot(Request::new(Body::empty())).await.unwrap();

        assert_eq!(res.headers()["content-type"], "text/markdown");

        let body = res.into_body().data().await.unwrap().unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.starts_with("# Tower Serve Static"));
    }

    #[tokio::test]
    async fn with_custom_chunk_size() {
        let svc = ServeFile::new(include_file!("/README.md")).with_buf_chunk_size(1024 * 32);

        let res = svc.oneshot(Request::new(Body::empty())).await.unwrap();

        assert_eq!(res.headers()["content-type"], "text/markdown");

        let body = res.into_body().data().await.unwrap().unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.starts_with("# Tower Serve Static"));
    }

    #[tokio::test]
    async fn with_mime() {
        let svc = ServeFile::new(include_file_with_mime!(
            "./README.md",
            mime::APPLICATION_OCTET_STREAM
        ));

        let res = svc.oneshot(Request::new(Body::empty())).await.unwrap();

        assert_eq!(res.headers()["content-type"], "application/octet-stream");

        let body = res.into_body().data().await.unwrap().unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.starts_with("# Tower Serve Static"));
    }

    // 404 is not possible with include_file!
    //
    // #[tokio::test]
    // async fn returns_404_if_file_doesnt_exist() {
    //     let svc = ServeFile::new(include_file!("/this-doesnt-exist.md"));
    //
    //     let res = svc.oneshot(Request::new(Body::empty())).await.unwrap();
    //
    //     assert_eq!(res.status(), StatusCode::NOT_FOUND);
    //     assert!(res.headers().get(header::CONTENT_TYPE).is_none());
    // }
}
