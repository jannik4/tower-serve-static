macro_rules! opaque_body {
    ($(#[$m:meta])* pub type $name:ident = $actual:ty;) => {
        opaque_body! {
            $(#[$m])* pub type $name<> = $actual;
        }
    };

    ($(#[$m:meta])* pub type $name:ident<$($param:ident),*> = $actual:ty;) => {
        #[pin_project::pin_project]
        $(#[$m])*
        pub struct $name<$($param),*>(#[pin] pub(crate) $actual);

        impl<$($param),*> http_body::Body for $name<$($param),*> {
            type Data = <$actual as http_body::Body>::Data;
            type Error = <$actual as http_body::Body>::Error;

            #[inline]
            fn poll_frame(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
                self.project().0.poll_frame(cx)
            }

            #[inline]
            fn is_end_stream(&self) -> bool {
                http_body::Body::is_end_stream(&self.0)
            }

            #[inline]
            fn size_hint(&self) -> http_body::SizeHint {
                http_body::Body::size_hint(&self.0)
            }
        }
    };
}
