#[async_trait]
trait TodoRepo {
    async fn get_all(&self) -> Vec<Todo>;

    async fn create(&self, title: String, description: String) -> Todo;

    async fn get(&self, id: i32) -> Option<Todo>;

    async fn update(&self, id: i32, title: Option<String>, description: Option<String>, done: Option<bool>) -> ();

    async fn delete(&self, id: i32) -> ();
}

#[derive(Debug, Clone)]
struct TodoRepoPostgres {
    pool: Pool<Postgres>
}

#[async_trait]
impl TodoRepo for TodoRepoPostgres {

}