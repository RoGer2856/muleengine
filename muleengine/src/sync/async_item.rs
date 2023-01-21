use std::{ops::Deref, sync::Arc};

use tokio::sync::{Notify, RwLock, RwLockReadGuard};

pub struct AsyncItem<T: Send> {
    value: Arc<RwLock<Option<T>>>,
    // todo!("use an async condvar")
    notify: Arc<Notify>,
}

impl<T: Send> Clone for AsyncItem<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            notify: self.notify.clone(),
        }
    }
}

pub struct AsyncItemReadGuard<'a, T: Send> {
    inner: RwLockReadGuard<'a, Option<T>>,
}

impl<T: Send> Deref for AsyncItemReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if let Some(value) = self.inner.as_ref() {
            value
        } else {
            unreachable!()
        }
    }
}

impl<T: Send> Default for AsyncItem<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send> AsyncItem<T> {
    pub fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(None)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn set(&self, value: T) {
        *self.value.write().await = Some(value);
        self.notify.notify_waiters();
    }

    pub async fn read(&self) -> AsyncItemReadGuard<T> {
        loop {
            let value_guard = self.value.read().await;
            if value_guard.is_some() {
                break AsyncItemReadGuard { inner: value_guard };
            }

            let notify_task = self.notify.notified();
            drop(value_guard);
            notify_task.await;
        }
    }
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};

    use tokio::time::sleep;

    use super::AsyncItem;

    #[tokio::test(flavor = "multi_thread")]
    async fn set_then_multiple_get() {
        let item = AsyncItem::new();

        item.set(7).await;

        assert_eq!(7, *item.read().await);
        assert_eq!(7, *item.read().await);
        assert_eq!(7, *item.read().await);
        assert_eq!(7, *item.read().await);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn multiple_gets_then_set() {
        let item = Arc::new(AsyncItem::new());

        let mut tasks = Vec::new();
        for _ in 0..100 {
            let item = item.clone();
            tasks.push(tokio::spawn(async move {
                assert_eq!(7, *item.read().await);
            }));
        }

        sleep(Duration::from_millis(200)).await;

        item.set(7).await;

        let join_task = async move {
            for task in tasks {
                task.await.unwrap();
            }
        };

        tokio::select! {
            _ = sleep(Duration::from_secs(2)) => {
                assert!(false);
            }
            _ = join_task => {
            }
        }
    }
}
