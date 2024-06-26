#![allow(dead_code)]
#![allow(unreachable_code)]

//!
//! CONTEXT
//! -------
//!
//! So far, you have seen context-free web applications in Rust. These web applications
//! do not share context with a higher level or between themselves.
//!
//! While appropriate for very simple applications, most real world applications will need
//! some form of context. For example, a web application might need to access a database,
//! and it would be inefficient to open a new connection to the database for every request.
//! So most handlers will end up drawing from a database connection pool.
//!
//! Axum has been designed to facilitate sharing context, both between handlers, and
//! between handlers and higher levels of the application.
//!
//! In this section, you will explore these mechanisms.
//!

use std::{collections::HashMap, sync::Arc};

#[allow(unused_imports)]
use axum::extract::State;
#[allow(unused_imports)]
use axum::{body::Body, http::Method, routing::*};
use axum::{extract::Path, response::IntoResponse, Json};
#[allow(unused_imports)]
use hyper::Request;
use hyper::{Response, StatusCode};
use tokio::sync::Mutex;

///
/// EXERCISE 1
///
/// While not a highly maintainable solution, it is possible to create contextual
/// web applications by using closures to capture context.
///
/// In this exercise, share the same `usd_to_gbp` rate between the two routes
/// by using closures.
///
#[tokio::test]
async fn closure_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    // this is an example of how to use closures to share context.
    let _gbp_to_usd_rate = 1.3;

    let _app = Router::<()>::new()
        .route(
            "/usd_to_gbp",
            get(move |usd: String| async move {
                // The first instance of "move" moves the closure into the async block. This is necessary because the closure
                // is a separate type from the async block, and the async block needs to own the closure. The second instance of
                // "move" moves the variables into the closure. This is necessary because the closure will be called multiple times,
                // and the variables need to be available each time.
                convert_usd_to_gbp(usd, _gbp_to_usd_rate)

                // if any variable in here doesn't implement Clone, the closure won't implement Clone, which means
                // it can't be used in the route. This is why the _gbp_to_usd_rate is moved into the closure. F64 does implement Clone.

                // Note: This is *not* an async closure, it just happens to have an async block inside it.
                // Eventually, Rust will support entire async closures, but for now, you have to use this pattern.

                // 💁‍♀️ If the function wasn't async, you wouldn't need the second "move" keyword at all.
            }),
        )
        .route(
            "/gbp_to_usd",
            get(move |gbp: String| async move { convert_gbp_to_usd(gbp, _gbp_to_usd_rate) }),
        );

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}

fn convert_usd_to_gbp(usd: String, gbp_to_usd_rate: f64) -> String {
    format!("{}", usd.parse::<f64>().unwrap() * gbp_to_usd_rate)
}
fn convert_gbp_to_usd(gbp: String, gbp_to_usd_rate: f64) -> String {
    format!("{}", gbp.parse::<f64>().unwrap() / gbp_to_usd_rate)
}

///
/// EXERCISE 2
///
/// The previous exercise was almost too easy, because the context was of type
/// `f64`, which is `Copy`. This means that the context was copied into both
/// closures, rather than truly shared between them.
///
/// Of course, for any data type that you do not wish to mutate, you can always
/// implement `Clone`, and then manually clone the context into each closure.
///
/// But what if you want to share a mutable context between handlers?
///
/// In this exercise, you will share a mutable context between handlers.
/// Specifically, you will share a mutably editable exchange rate between
/// GBP and USD currencies. Consider using the `Arc` type, which you will
/// have to use atop Tokio's Mutex in order to support mutation.
///
/// When you are done, try to generalize what you have learned about sharing
/// context between handlers. What would you use if the context were
/// immutable? What would you use if the context were mutable?
///
#[tokio::test]
async fn shared_mutable_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    use tokio::sync::Mutex;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    // Docs on how Arc keeps track of how many references there are in it:
    // https://doc.rust-lang.org/std/sync/struct.Arc.html

    // Mutex is only required if you need to be able to update information in a concurrent-safe way.
    // If the value being shared is read-only, you don't need a Mutex, just an Arc.
    let arc = Arc::new(Mutex::new(1.3));

    // LOL this is janky. We have to clone the Arc because we can't move it into the closure.
    // But arc keeps track of how many clones there are, and when the last clone goes out of scope, it drops the value inside.
    let arc2 = arc.clone();

    // Mutex lets us share mutable state between handlers,
    // and Arc lets us change the state in one handler and see the change in another handler.

    // because our datatime implements Clone, sharing here is a piece of cake 🍰

    let _app = Router::<()>::new()
        .route(
            "/usd_to_gbp",
            get(move |usd: String| async move {
                // this gives you access to the thing *inside* the arc, which is the Mutex.
                // lock() returns a Future.
                // This lock is async because someone else might be holding the lock, and you have to wait for them to release it.
                // gotta wait in line to get the lock.
                let guard = arc.lock().await;

                // when guard goes out of scope, the lock is released.
                // here we have to dereference the guard to get the value inside the Mutex with "*"
                convert_usd_to_gbp(usd, *guard)
            }),
        )
        .route(
            "/gbp_to_usd",
            get(move |gbp: String| async move {
                let guard = arc2.lock().await;
                convert_gbp_to_usd(gbp, *guard)
            }),
        );

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}

///
/// EXERCISE 3
///
/// Having to write all your handlers as closures is not very ergonomic, and could
/// lead to either boilerplate or gigantic functions that define all handlers.
///
/// Instead, Axum provides direct support for sharing context. This shared context
/// can be specified in your Router, and it can be passed into your handlers as
/// a State parameter.
///
/// In this exercise, share the same `usd_to_gbp` rate between the two routes
/// by using the `State` extractor, defined in `axum::extract`. Note that you
/// will have to supply the state by using the `.with_state` method on your
/// Router. An example (using () as the state type) has been provided below.
///
#[tokio::test]
async fn state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _gbp_to_usd_rate = 1.3;

    let _app = Router::new()
        .route("/usd_to_gbp", get(usd_to_gbp_handler))
        .route("/gbp_to_usd", get(gbp_to_usd_handler))
        .with_state(_gbp_to_usd_rate);
    // way easier to share state with this ^

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn usd_to_gbp_handler(State(rate): State<f64>, body: String) -> String {
    let body_as_f64 = body.parse::<f64>().unwrap();

    (rate * body_as_f64).to_string()
}
async fn gbp_to_usd_handler(State(rate): State<f64>, body: String) -> String {
    let body_as_f64 = body.parse::<f64>().unwrap();
    (rate * body_as_f64).to_string()
}

///
/// EXERCISE 4
///
/// Now that you have seen Axum's first-class support for context sharing, it's
/// time to leverage your knowledge of Rust to enable sharing mutable context
/// between handlers, building upon what you have done in previous exercises.
///
/// Modify this exercise to share a mutable exchange rate between GBP and USD.
///
#[tokio::test]
async fn mutable_state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let gbp_to_usd_rate = Arc::new(Mutex::new(1.3));

    let _app = Router::new()
        .route("/usd_to_gbp", get(mutable_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(mutable_gbp_to_usd_handler))
        .route("/set_exchange_rate", post(set_mutable_gbp_to_usd_handler))
        .with_state(gbp_to_usd_rate);

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn mutable_usd_to_gbp_handler(State(rate): State<Arc<Mutex<f64>>>, body: String) -> String {
    let body_as_f64 = body.parse::<f64>().unwrap();

    let guard = rate.lock().await;

    (*guard * body_as_f64).to_string()
}
async fn mutable_gbp_to_usd_handler(State(rate): State<Arc<Mutex<f64>>>, body: String) -> String {
    let body_as_f64 = body.parse::<f64>().unwrap();

    let guard = rate.lock().await;

    (*guard * body_as_f64).to_string()
}
async fn set_mutable_gbp_to_usd_handler(State(rate): State<Arc<Mutex<f64>>>, body: String) -> () {
    let body_as_f64 = body.parse::<f64>().unwrap();
    println!("body_as_f64: {}", body_as_f64);

    // this let's use update the value inside the Mutex.
    let mut guard = rate.lock().await;

    // here we assign the new value to the guard.
    *guard = body_as_f64
}

///
/// EXERCISE 5
///
/// The type `S` flows through a lot of the types in Axum (Router, MethodRouter,
/// Handler, etc.). If you examine closely the signatures for methods that combine
/// routers, you will see that their state types have to be exactly the same.
///
/// What happens if your handlers, from different parts of your application,
/// require totally different state?
///
/// One possible solution to this problem is to make your handlers polymorphic
/// in the type of state they handle, and to use traits that expose "accessors"
/// for the specific state type they require.
///
/// In this exercise, you will use this technique to complete the following
/// exercise.
///
/// Assume that some handlers require state type `GBPtoUSD`, and that other
/// handlers require state type `EURtoUSD`. Further, assume you have a
/// composite state type, `AllExchangeRates`, that contains both `GBPtoUSD`
/// and `EURtoUSD`.
///
/// Invent traits that can describe what each type of handler requires from
/// the "global state", and then make the handlers polymorphic in the state
/// type, requiring only an implementation of the appropriate trait.
///
/// You might have to supply some type hints to the compiler in order to
/// construct the routes with your polymorphic handlers.
///
/// This technique is very powerful, and it can allow state to vary across
/// a modular web application, where different types of endpoints have
/// different requirements for context.
///
#[tokio::test]
async fn generic_state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _app = Router::new()
        .route("/usd_to_gbp", get(generic_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(generic_gbp_to_usd_handler))
        .route("/eur_to_usd", get(generic_eur_to_usd_handler))
        .route("/usd_to_eur", get(generic_usd_to_eur_handler))
        .with_state(AllExchangeRates {
            gbp_to_usd: GBPtoUSD(1.3),
            eur_to_usd: EURtoUSD(1.2),
        });

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn generic_usd_to_gbp_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_gbp_to_usd_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_eur_to_usd_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_usd_to_eur_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct AllExchangeRates {
    gbp_to_usd: GBPtoUSD,
    eur_to_usd: EURtoUSD,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct GBPtoUSD(f64);
#[derive(Clone, Copy, Debug, PartialEq)]
struct EURtoUSD(f64);

///
/// EXERCISE 6
///
/// Although it is possible to share virtually any kind of context using State,
/// with the appropriate type classes and polymorphic handlers allowing state
/// to vary across a web application, some would prefer to reduce the amount of
/// ceremony required to share varying context, and are willing to accept a
/// tradeoff in terms of static type safety.
///
/// For this audience, Axum has a solution called Extensions. Extensions can be
/// used to share context between middleware and handlers, or just to share
/// context either between handlers, between middleware, or between either
/// handlers or middleware and higher levels of the application.
///
/// In order to use extensions, your handler may require a parameter of type
/// `axum::extract::Extension<T>` where `T` is the type of the context you
/// wish to share. Then you must install a layer in your router, which holds
/// the context, and you can do that with the `Extension(...)` constructor.
///
/// In this exercise, you will implement the same exchange-rate-sharing
/// application, but this time using an extension to share state.
///
/// Experiment with what happens when you forget to install the extension.
/// Under what circumstances would you prefer extensions to state for
/// sharing context? Under what circumstances would you prefer the reverse?
///
#[tokio::test]
async fn extension_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _gbp_to_usd_rate = 1.3;

    let _app = Router::new()
        .route("/usd_to_gbp", get(extension_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(extension_gbp_to_usd_handler));

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn extension_usd_to_gbp_handler() -> String {
    todo!("Use Extensions to access the exchange rate")
}
async fn extension_gbp_to_usd_handler() -> String {
    todo!("Use Extensions to access the exchange rate")
}

///
/// GRADUATION PROJECT
///
/// Provide a complete implementation of the following API, which uses shared mutable
/// state across all the handlers to provide a fake implementation of the full CRUD
/// API.
///
/// GET /users
/// GET /users/:id
/// POST /users
/// PUT /users/:id
/// DELETE /users/:id
///
/// Place it into a web server and test to ensure it meets your requirements.
///
pub async fn run_users_server() {
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(UsersState::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(state): State<UsersState>) -> Json<Vec<User>> {
    Json(state.get_users().await)
}
async fn get_user(
    State(state): State<UsersState>,
    Path(id): Path<u64>,
) -> Result<Json<User>, MissingUser> {
    match state.get_user(id).await {
        Some(user) => Ok(Json(user)),
        None => Err(MissingUser { id }),
    }
}
async fn create_user(
    State(state): State<UsersState>,
    Json(create_request): Json<UserWithoutId>,
) -> Json<CreateUserResponse> {
    let id = state.create_user(create_request).await;

    Json(CreateUserResponse { id })
}
async fn update_user(
    State(state): State<UsersState>,
    Path(id): Path<u64>,
    Json(update_request): Json<UpdateUserRequest>,
) -> Result<(), MissingUser> {
    let result = state.update_user(id, update_request).await;

    result.map_err(|missing_user| missing_user)
}
async fn delete_user(
    State(state): State<UsersState>,
    Path(id): Path<u64>,
) -> Result<(), MissingUser> {
    let result = state.delete_user(id).await;

    result.map_err(|missing_user| missing_user)
}

#[derive(Clone)]
struct UsersState {
    users: Arc<Mutex<HashMap<u64, UserWithoutId>>>,
    counter: Arc<Mutex<u64>>,
}

impl UsersState {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn get_users(&self) -> Vec<User> {
        let guard = self.users.lock().await;

        (*guard)
            .iter()
            .map(|(id, user)| User {
                id: *id,
                name: user.name.clone(),
                email: user.email.clone(),
            })
            .collect()
    }

    async fn get_user(&self, id: u64) -> Option<User> {
        let guard = self.users.lock().await;

        guard.get(&id).map(|user| User {
            id,
            name: user.name.clone(),
            email: user.email.clone(),
        })
    }

    async fn create_user(&self, user: UserWithoutId) -> u64 {
        let mut guard = self.users.lock().await;

        let id = {
            let mut counter_guard = self.counter.lock().await;
            *counter_guard += 1;
            *counter_guard
        };
        guard.insert(id, user);
        id
    }

    async fn update_user(&self, id: u64, update: UpdateUserRequest) -> Result<(), MissingUser> {
        let mut guard = self.users.lock().await;

        if let Some(user) = guard.get_mut(&id) {
            if let Some(name) = update.name {
                user.name = name;
            }

            if let Some(email) = update.email {
                user.email = email;
            }

            Ok(())
        } else {
            Err(MissingUser { id })
        }
    }

    async fn delete_user(&self, id: u64) -> Result<(), MissingUser> {
        let mut guard = self.users.lock().await;

        if guard.remove(&id).is_some() {
            Ok(())
        } else {
            Err(MissingUser { id })
        }
    }
}

impl IntoResponse for MissingUser {
    fn into_response(self) -> axum::http::Response<Body> {
        let response = MissingUserErrorDetails {
            id: self.id,
            message: format!("User with id {} not found", self.id),
        };

        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(serde_json::to_string(&response).unwrap()))
            .unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct MissingUserErrorDetails {
    id: u64,
    message: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct UserWithoutId {
    name: String,
    email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateUserResponse {
    id: u64,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct MissingUser {
    id: u64,
}
