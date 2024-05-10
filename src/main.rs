mod architecture;
mod basics;
mod client;
mod context;
mod handlers;
mod middleware;
mod persistence;
mod playground;
mod welcome;

#[tokio::main]
async fn main() {
    // playground::example_postgres().await.unwrap();
    // basics::hello_world().await;
    // context::run_users_server().await;

    // client::cat_fact_server().await;
    // client::posts_server().await;
    // client::graduation_project().await; // this is the wine server I made 🍷
    persistence::run_todo_app().await;

    println!("Hello, world!");
}
