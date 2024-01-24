//! Tower file services embedding assets into the binary.
//!
//! # Serve Static File
//!
//! ```
//! use tower_serve_static::{ServeFile, include_file};
//!
//! // File is located relative to `CARGO_MANIFEST_DIR` (the directory containing the manifest of your package).
//! // This will embed and serve the `README.md` file.
//! let service = ServeFile::new(include_file!("/README.md"));
//!
//! // Run our service using `axum`
//! let app = axum::Router::new().nest_service("/", service);
//!
//! # async {
//! // run our app with axum, listening locally on port 3000
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
//! axum::serve(listener, app).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # };
//! ```
//!
//! # Serve Static Directory
//!
//! ```
//! use tower_serve_static::{ServeDir};
//! use include_dir::{Dir, include_dir};
//!
//! // Use `$CARGO_MANIFEST_DIR` to make path relative to your package.
//! // This will embed and serve files in the `src` directory and its subdirectories.
//! static ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src");
//! let service = ServeDir::new(&ASSETS_DIR);
//!
//! // Run our service using `axum`
//! let app = axum::Router::new().nest_service("/", service);
//!
//! // run our app with axum, listening locally on port 3000
//! # async {
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
//! axum::serve(listener, app).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # };
//! ```

#[macro_use]
mod macros;

mod serve_dir;
mod serve_file;

#[doc(hidden)]
pub mod private {
    pub use {http, mime, mime_guess};
}

use bytes::Bytes;
use http_body::{Body, Frame};
use pin_project::pin_project;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncRead;

use futures_util::Stream;
use tokio_util::io::ReaderStream;

// default capacity 64KiB
const DEFAULT_CAPACITY: usize = 65536;

pub use self::{
    serve_dir::{
        ResponseBody as ServeDirResponseBody, ResponseFuture as ServeDirResponseFuture, ServeDir,
    },
    serve_file::{
        File, ResponseBody as ServeFileResponseBody, ResponseFuture as ServeFileResponseFuture,
        ServeFile,
    },
};

// NOTE: This could potentially be upstreamed to `http-body`.
/// Adapter that turns an `impl AsyncRead` to an `impl Body`.
#[pin_project]
#[derive(Debug)]
pub struct AsyncReadBody<T> {
    #[pin]
    reader: ReaderStream<T>,
}

impl<T> AsyncReadBody<T>
where
    T: AsyncRead,
{
    /// Create a new [`AsyncReadBody`] wrapping the given reader,
    /// with a specific read buffer capacity
    fn with_capacity(read: T, capacity: usize) -> Self {
        Self {
            reader: ReaderStream::with_capacity(read, capacity),
        }
    }
}

impl<T> Body for AsyncReadBody<T>
where
    T: AsyncRead,
{
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.project().reader.poll_next(cx).map(|res| match res {
            Some(Ok(buf)) => Some(Ok(Frame::data(buf))),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        })
    }
}

#[cfg(feature = "metadata")]
fn unmodified_since_request_condition<T>(file: &include_dir::File, req: &http::Request<T>) -> bool {
    use http::{header, Method};
    use httpdate::HttpDate;

    let Some(metadata) = file.metadata() else {
        return false;
    };

    // When used in combination with If-None-Match, it is ignored, unless the server doesn't support If-None-Match.
    if req.headers().contains_key(header::IF_NONE_MATCH) {
        return false;
    }

    // If-Modified-Since can only be used with a GET or HEAD.
    match req.method() {
        &Method::GET | &Method::HEAD => (),
        _ => return false,
    }

    let Some(since) = req
        .headers()
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<HttpDate>().ok())
    else {
        return false;
    };

    metadata.modified() <= since.into()
}
