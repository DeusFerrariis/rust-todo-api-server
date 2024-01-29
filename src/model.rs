pub trait RecordStore<R, I> {
    type Error;
    type Result<T> = Result<T, Self::Error>;

    async fn create_record(&self, record: R) -> Self::Result<I>;
    async fn delete_record(&self, record_id: I) -> Self::Result<Option<R>>;
    async fn get_record(&self, record_id: I) -> Self::Result<Option<R>>;
    async fn get_records(
        &self,
        start: usize,
        count: usize,
    ) -> Self::Result<Vec<(usize, String)>>;
    async fn patch_record(
        &self,
        record_id: I,
        patch: impl Fn(R) -> Self::Result<R>,
    ) -> Self::Result<R>;
}