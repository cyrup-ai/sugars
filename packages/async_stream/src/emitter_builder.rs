//! EmitterBuilder - builds AsyncStream with error handling

#[cfg(all(
    feature = "crossbeam-backend",
    not(feature = "tokio-backend"),
    not(feature = "std-backend")
))]
use crate::stream::crossbeam::AsyncStream;
#[cfg(all(feature = "std-backend", not(feature = "tokio-backend")))]
use crate::stream::std::AsyncStream;
#[cfg(feature = "tokio-backend")]
use crate::stream::tokio::AsyncStream;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;

/// Builder that emits AsyncStream after handling Result
pub struct EmitterBuilder<T> {
    inner: Box<dyn EmitterImpl<T>>,
}

/// Type alias for complex future return type
pub type EmitterFuture<T> =
    Pin<Box<dyn Future<Output = Result<Vec<T>, Box<dyn std::error::Error + Send>>> + Send>>;

/// Hidden trait for implementation
pub trait EmitterImpl<T>: Send {
    /// Executes the emitter and returns a future that resolves to a vector of results.
    fn execute(self: Box<Self>) -> EmitterFuture<T>;
}

impl<T: Send + 'static + super::NotResult> EmitterBuilder<T> {
    /// Create a new EmitterBuilder
    pub fn new(inner: Box<dyn EmitterImpl<T>>) -> Self {
        Self { inner }
    }

    /// Execute with error handling
    pub fn emit<FOk, FErr>(self, on_ok: FOk, on_err: FErr) -> AsyncStream<T>
    where
        FOk: FnOnce(Vec<T>) -> Vec<T> + Send + 'static,
        FErr: FnOnce(Box<dyn std::error::Error + Send>) + Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            match self.inner.execute().await {
                Ok(items) => {
                    for item in on_ok(items) {
                        if tx.send(item).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => on_err(e),
            }
        });

        AsyncStream::new(rx)
    }
}

/// Macro for cleaner emit syntax
#[macro_export]
macro_rules! emit {
    ($stream:expr, Ok($items:ident) => $ok:expr, Err($e:ident) => $err:expr) => {
        $stream.emit(|$items| $ok, |$e| $err)
    };
}
