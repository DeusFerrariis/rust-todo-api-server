pub mod task {
    // Task Handlers
    use super::super::*;
    use crate::store::local::MutexVec;
    use crate::model::RecordStore;
    use rocket::serde::json::Json;
    use rocket::State;

    #[rocket::post("/task", data = "<payload>")]
    pub async fn create_task(
        store: &State<MutexVec<(usize, String)>>,
        payload: Json<NewTaskRequest>,
    ) -> Result<Json<TaskResponse>, Status> {
        let content = payload.0.content;
        let id = store
            .inner
            .create_record(content.clone())
            .await
            .map_err(|_| Status::InternalServerError)?;

        Ok(Json(TaskResponse { id, content }))
    }

    #[rocket::get("/task/<id>")]
    pub async fn get_task(
        store: &State<MutexVec<(usize, String)>>,
        id: usize,
    ) -> Result<Json<TaskResponse>, Status> {
        let result = store
            .inner
            .get_record(id)
            .await
            .map_err(|_| Status::InternalServerError)?;

        match result {
            Some(s) => Ok(Json(TaskResponse {
                id: id,
                content: s
            })),
            None => Err(Status::NotFound)
        }
    }

    #[rocket::put("/task/<id>/complete")]
    pub async fn complete_task(
        store: &State<MutexVec<(usize, String)>>,
        id: usize,
    ) -> Result<Json<TaskResponse>, Status> {
        let result = store
            .inner
            .delete_record(id)
            .await
            .map_err(|_| Status::InternalServerError)?;

        match result {
            Some(s) => Ok(Json(TaskResponse {
                id,
                content: s,
            })),
            None => Err(Status::NotFound),
        }
    }

    #[rocket::get("/tasks")]
    pub async fn get_tasks(
        store: &State<MutexVec<(usize, String)>>,
    ) -> Result<Json<Vec<TaskResponse>>, Status> {
        let result = store
            .inner
            .get_records(0, 10)
            .await
            .map_err(|_| Status::InternalServerError)?;

        let mapped = result
            .iter()
            .cloned()
            .map(|(i, s)| {
                TaskResponse {
                    id: i,
                    content: s,
                }
            })
            .collect();

        Ok(Json(mapped))
    }
}
