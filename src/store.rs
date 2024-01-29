use std::sync::Arc;
use tokio::sync::Mutex;

use crate::model::RecordStore;

pub mod local {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub struct MutexVec<T> {
        pub inner: Arc<Mutex<Vec<T>>>,
    }

    impl<T> MutexVec<T> {
        pub fn new() -> Self {
            Self {
                inner: Arc::new(Mutex::new(Vec::<T>::new())),
            }
        }
    }
}

impl RecordStore<String, usize> for Arc<Mutex<Vec<(usize, String)>>> {
    type Error = ();

    type Result<T> = Result<T, Self::Error>;

    async fn create_record(&self, record: String) -> Self::Result<usize> {
        let new_id: usize = {
            let lock = self.lock().await;

            match lock.last() {
                Some(lt) => lt.0 + 1,
                _ => 0,
            }
        };

        let mut lock = self.lock().await;
        let _ = lock.push((new_id, record));

        Ok(new_id)
    }

    async fn delete_record(&self, record_id: usize) -> Self::Result<Option<String>> {
        let mut lock = self.lock().await;
        let mut found: Option<String> = None;

        lock.retain(|(i, t)| {
            if i == &record_id {
                found = Some(t.to_string());
            }

            i != &record_id
        });

        Ok(found)
    }

    async fn get_record(&self, record_id: usize) -> Self::Result<Option<String>> {
        let lock = self.lock().await;
        let mut found: Option<String> = None;

        if let Some((_, t)) = lock.iter().find(|(i, _)| i == &record_id) {
            found = Some(t.to_string());
        }

        Ok(found)
    }

    async fn patch_record(
        &self,
        record_id: usize,
        patch: impl Fn(String) -> Self::Result<String>,
    ) -> Self::Result<String> {
        let result = self.get_record(record_id).await;
        let Ok(Some(r)) = result else {
            return Err(());
        };

        let Ok(p) = patch(r) else { return Err(()) };

        {
            let lock = self.lock().await;
            let _ = lock
                .iter()
                .map(|(i, t)| if i == &record_id { (i, &p) } else { (i, t) });
        }

        Ok(p)
    }

    async fn get_records(
        &self,
        start: usize,
        count: usize,
    ) -> Self::Result<Vec<(usize, String)>> {
        let lock = self.lock().await;
        let results: Vec<(usize, String)> = lock
            .iter()
            .skip(start)
            .take(count)
            .cloned()
            .collect();

        Ok(results)
    }
}