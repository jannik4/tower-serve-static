//! Tower file services embedding assets into the binary.
//!
//! # Serve Static File
//!
//! ```
//! use tower_serve_static::{ServeFile, include_file};
//!
//! // File is located relative to `CARGO_MANIFEST_DIR` (the directory containing the manifest of your package).
//! // This will embed and serve the `README.md` file.
//! let service = ServeFile::new(include_file!("./README.md"));
//!
//! # async {
//! // Run our service using `hyper`
//! let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
//! hyper::Server::bind(&addr)
//!     .serve(tower::make::Shared::new(service))
//!     .await
//!     .expect("server error");
//! # };
//! ```
//!
//! # Serve Static Directory
//!
//! ```
//! use tower_serve_static::{ServeDir, Dir, include_dir};
//!
//! // Use `$CARGO_MANIFEST_DIR` to make path relative to your package.
//! // This will embed and serve files in the `src` directory and its subdirectories.
//! static ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src");
//! let service = ServeDir::new(&ASSETS_DIR);
//!
//! # async {
//! // Run our service using `hyper`
//! let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
//! hyper::Server::bind(&addr)
//!     .serve(tower::make::Shared::new(service))
//!     .await
//!     .expect("server error");
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
use http::HeaderMap;
use http_body::Body;
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
        include_dir, Dir, ResponseBody as ServeDirResponseBody,
        ResponseFuture as ServeDirResponseFuture, ServeDir,
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

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().reader.poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
