#![feature(associated_type_defaults)]

use std::sync::Arc;

use tokio::sync::Mutex;

pub trait RecordStore<R, I> {
    type Error;
    type Result<T> = Result<T, Self::Error>;

    async fn create_record(&self, record: R) -> Self::Result<I>;
    async fn delete_record(&self, record_id: I) -> Self::Result<Option<R>>;
    async fn get_record(&self, record_id: I) -> Self::Result<Option<R>>;
    async fn patch_record(&self, record_id: I, patch: impl Fn(R) -> Self::Result<R>) -> Self::Result<R>;
}

pub struct MutexVec<T> {
    inner: Arc<Mutex<Vec<T>>>
}

impl<T> MutexVec<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::<T>::new())),
        }
    }
}

impl RecordStore<String, usize> for MutexVec<(usize, String)> {
    type Error = ();

    type Result<T> = Result<T, Self::Error>;

    async fn create_record(&self, record: String) -> Self::Result<usize> {

        let new_id: usize = {
            let lock = self.inner.lock().await;

            match lock.last() {
                Some(lt) => lt.0 + 1,
                _ => 0,
            }
        };

        let mut lock = self.inner.lock().await;
        let _ = lock.push((new_id, record));

        Ok(new_id)
    }

    async fn delete_record(&self, record_id: usize) -> Self::Result<Option<String>> {
        let mut lock = self.inner.lock().await;
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
        let lock = self.inner.lock().await;
        let mut found: Option<String> = None;

        if let Some((_, t)) = lock
            .iter()
            .find(|(i, _)| i == &record_id) {
            found = Some(t.to_string());
        }

        Ok(found)
    }

    async fn patch_record(
        &self,
        record_id: usize,
        patch: impl Fn(String) -> Self::Result<String>
    ) -> Self::Result<String> {

        let result = self.get_record(record_id).await;
        let Ok(Some(r)) = result else {
            return Err(());
        };
        
        let Ok(p) = patch(r) else {
            return Err(())
        };

        {
            let lock = self.inner.lock().await;
            let _ = lock.iter().map(|(i, t)| {
                if i == &record_id {
                    (i, &p)
                } else {
                    (i, t)
                }
            });
        }

        Ok(p)
    }
}

#[derive(Serialize, Deserialize)]
pub struct TaskData {
    id: usize,
    content: String,
}

// TODO: create_task
// TODO: get_task
// TODO: get_tasks
// TODO: complete_task

#[tokio::main]
async fn main() {
}

#[cfg(test)]
mod test {
    use crate::MutexVec;
    use crate::RecordStore;

    #[tokio::test]
    async fn test_mutex_store() {
        let store: MutexVec<(usize, String)> = MutexVec::<(usize, String)>::new();

        // Create a record
        let result = store.create_record(String::from("Make dinner")).await;
        assert!(result == Ok(0));

        // Get a record
        let get_result = store.get_record(0).await;
        assert!(get_result == Ok(Some(String::from("Make dinner"))));

        // Delete a record
        let delete_result = store.delete_record(0).await;
        assert!(delete_result != Err(()));
        assert_eq!(delete_result, Ok(Some(String::from("Make dinner"))));

        // Create after delete
        let _ = store.create_record(String::from("Make dinner")).await; // 0
        let _ = store.create_record(String::from("Make dinner  ")).await; // 1
        assert!(store.inner.lock().await.len() == 2);

        let result = store.create_record(String::from("Make dinner")).await; // 1
        assert!(result == Ok(2));

        // Update a record
        let result = store.patch_record(1, |_| {
            Ok(String::from("Clean up dinner"))
        }).await;

        assert_eq!(result, Ok(String::from("Clean up dinner")));
    }
}