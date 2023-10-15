use futures::Future;

const WORKER_THREADS: usize = 2;

pub struct MyExecutor {
    rt: tokio::runtime::Runtime,
}

impl iced_futures::Executor for MyExecutor {
    fn new() -> Result<Self, futures::io::Error> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKER_THREADS)
            .enable_all()
            .build()?;

        Ok(MyExecutor { rt })
    }

    #[allow(clippy::let_underscore_future)]
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = self.rt.spawn(future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.rt.enter();
        f()
    }
}
