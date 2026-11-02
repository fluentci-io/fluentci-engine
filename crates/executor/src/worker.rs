//! Owned background work: cancellation and joining precede resource teardown.
use crate::Cancellation;
use std::thread::{self, JoinHandle};

pub(crate) struct Worker<T> {
    cancel: Cancellation,
    handle: Option<JoinHandle<Result<T, String>>>,
}
impl<T: Send + 'static> Worker<T> {
    pub fn spawn(
        work: impl FnOnce(Cancellation) -> Result<T, String> + Send + 'static,
    ) -> Result<Self, String> {
        let cancel = Cancellation::default();
        let token = cancel.clone();
        let handle = thread::Builder::new()
            .name("fluentci-cache-save".into())
            .spawn(move || work(token))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            cancel,
            handle: Some(handle),
        })
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn poll(&mut self) -> Option<Result<T, String>> {
        if !self.handle.as_ref()?.is_finished() {
            return None;
        }
        Some(
            self.handle
                .take()
                .unwrap()
                .join()
                .unwrap_or_else(|_| Err("cache worker panicked".into())),
        )
    }
}
impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.cancel.cancel();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    #[test]
    fn pending_work_does_not_block_poll_and_drop_joins_after_cancellation() {
        let (started_tx, started_rx) = mpsc::channel();
        let (ended_tx, ended_rx) = mpsc::channel();
        let mut worker = Worker::spawn(move |cancel| {
            started_tx.send(()).unwrap();
            while !cancel.is_cancelled() {
                thread::yield_now();
            }
            ended_tx.send(()).unwrap();
            Ok(())
        })
        .unwrap();
        started_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert!(worker.poll().is_none());
        drop(worker);
        ended_rx.try_recv().unwrap();
    }
}
