#![allow(dead_code)]
// this is a "block attribute". We know that because of the "!", which means it's attached to the entire file.
#![allow(unreachable_code)]

//!
//! BASICS
//! ------
//!
//! Axum is a web application framework that focuses on ergonomics, modularity,
//! and performance.
//!
//! In this section, you will learn the basics of building web applications
//! using the Axum framework. Although many of the specifics that you learn
//! will be Axum-specific, the concepts that you learn will be applicable to
//! other web frameworks as well.
//!  

#[allow(unused_imports)]
// this is an attribute that applies to the next item in the file only.
use axum::{
    body::Body,
    http::{Method, Request},
    response::Html,
    routing::*,
    Json, Router,
}; // use is kind of like import, but it has an advantage of de-duplication and aliasing.

///
/// In this "hello world" example, you can see the core elements of an Axum
/// web application:
///
/// 1. A router, which is used for specifying routes.
/// 2. A single route, defined with a path and a handler.
/// 3. A handler, which is an asynchronous function that returns a response.
/// 4. A listener, which is used to listen for incoming connections.
/// 5. A call to `axum::serve`, which starts the server.
///
pub async fn hello_world() {
    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // merge other routes in file from other functions
    let merge_app = build_router(app);

    // The function route() looks like it's a method, but it's actually a function that returns a Router.
    // get() is a method from the Router that indicates this route will use the GET HTTP method.
    // This works in Rust because the first argument to a method is always a reference to the object itself.

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    // Unwrap should not be used in production code.
    // It's a way to quickly handle errors in examples, but it's not a good practice.

    println!("Listening on {}", listener.local_addr().unwrap());
    // Rust macros are like functions, but they're evaluated at compile time.
    // They are powerful because they can do things that functions can't, like generate code.

    axum::serve(listener, merge_app).await.unwrap();
}

///
/// EXERCISE 1
///
/// Use the `Html` constructor to construct an HTML response that includes the
/// text "Hello, World!".
///
/// Run the hello_world() function to ensure that you can browse the `/` route
/// and that it properly serves the static HTML.
///
async fn handler() -> Html<&'static str> {
    println!("in the hello_world handler");
    Html("<h1>🌎 Hello, World!</h1>")
}

///
/// EXERCISE 2
///
/// Add the following routes to `_router``, using the dummy_handler for each.
///
/// GET /users/
/// GET /users/:id
/// POST /users/
/// PUT /users/:id
/// DELETE /users/:id
///
///
/// // this function accepts any type "S", but it has to have methods of Clone, Send, Sync, and 'static.
/// // this is a polymorphic function, which means it can work with any type that satisfies the constraints.
/// The "<>" syntax is used to specify a generic type, which is a placeholder for a type that will be provided later.
fn build_router<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    // _router is not used in this function, but it's a good practice to keep it in the function signature.
    // Preceding an unused variable with an underscore is a convention in Rust to indicate that the variable is unused.
    router
        .route("/users", get(dummy_handler))
        .route("/users/:id", get(dummy_handler))
        .route("/users", post(dummy_handler))
        .route("/users/:id", put(dummy_handler))
        .route("/users/:id", delete(dummy_handler))
}

async fn dummy_handler() -> Html<&'static str> {
    println!("in the dummy handler");
    Html("<h1>🤪 Dummy Handler</h1>")
}

///
/// EXERCISE 2
///
/// Using Router::merge, combine two routers into one.
///
/// What are the semantics of the resulting router?
///
fn merge_routers<S: Clone + Send + Sync + 'static>(left: Router<S>, right: Router<S>) -> Router<S> {
    // 'static means that if S contains references,
    // they must have a lifetime of 'static, meaning they must live for the entire duration of the program.
    // aka if you get rid of the pointer, this type won't be able to live on its own.

    // This is an example of destructuring a tuple.
    // let (_, _) = (left, right);

    left.merge(right)
}

///
/// EXERCISE 3
///
/// To factor out duplication across route paths, you can use the `nest` method
/// on Router. This method takes a path prefix and a router, and returns a new
/// router that has the path prefix applied to all of the routes in the nested
/// router.
///
/// In the following example, use the `nest` method to nest all of the user
/// routes under the `/users` path prefix of the specified router.
///
fn nest_router<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    let user_routes = Router::<S>::new()
        .route("/", get(handler))
        .route("/:id", get(handler))
        .route("/", post(handler))
        .route("/:id", put(handler))
        .route("/:id", delete(handler));

    router.nest("/users", user_routes)
    // this puts the user_routes under the /users path prefix of the specified router.
}

///
/// EXERCISE 4
///
/// Being able to test your routes without spinning up a server is very important for
/// performance and determinism. Fortunately, Axum is built on Tower, which provides a
/// convenient way to test your routes (`oneshot`).
///
/// Use `Request::builder` to construct a `Request` that makes the following unit test
/// pass. Try to pay attention to how to use `oneshot` and which imports are needed and
/// for what reasons.
///
#[tokio::test]

// This version doesn't use ? to propagate errors up the call stack, which apparently is better for testing.
async fn test_routes() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let app = Router::new().route("/users", get(identity_handler));

    let req: Request<Body> = Request::builder()
        .method(Method::GET)
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_as_string, "/users");
}
///
///
#[tokio::test] // this is required because it's an async test
async fn test_routes_prod() -> Result<(), String> {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let app = Router::new().route("/users", get(identity_handler));

    let req: Request<Body> = Request::builder()
        .method(Method::GET)
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.map_err(|e| e.to_string())?;
    // the "?" operator is used to propagate errors up the call stack.
    // This is useful in production code because it makes error handling more concise.
    // To handle this error up the call stack, this function must return a "Result" type.

    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| e.to_string())?
        .to_bytes();

    let body_as_string = String::from_utf8(body.to_vec()).map_err(|e| e.to_string())?;

    assert_eq!(body_as_string, "/users");

    Ok(())
}

///
/// EXERCISE 5
///
/// Axum makes it easy for your handlers to return JSON responses. To do so, you
/// can use the `Json` wrapper type, which implements `From<T>` for any type `T`
/// that implements `serde::Serialize`.
///
/// Create a `struct` and be sure to derive Serialize, and then use your struct
/// in the following test and handler.
///
#[tokio::test]
async fn test_basic_json() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let app = Router::<()>::new().route("/users/jdoe", get(return_json_hello_world));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/users/jdoe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_as_string, "{}");
}
async fn return_json_hello_world() -> Json<Dummy> {
    // use serde_json::json! to create a JSON object
    Json(Dummy {})
}

// Serialize and Copy are traits that are automatically derived for a struct when it's created.
// Copy is a trait that allows a type to be copied by value, which means it can be cloned without using the heap, which is faster
// and means that you don't have to worry about ownership or borrowing.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)] // commented out in favor of manual implementation below.
struct Dummy {}

// DIY serialization for Dummy data type
// impl serde::Serialize for Dummy {
//     // for some reason, we decided to implement our own serializer lol
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         use serde::ser::SerializeStruct; // weird to use imports here but ok

//         let struct_serializer = serializer.serialize_struct("Dummy", 0)?;

//         struct_serializer.end()
//     }
// }

// notice that imple is separate from the struct definition because they work together to define the struct in an extensible way.
impl Dummy {
    fn new() -> Self {
        // creates constructor for Dummy
        println!("I made a dummy!");
        Dummy {} // returns the struct "Dummy"
    }
    fn to_string(&self) -> String {
        // creates a method for Dummy
        let _ = "test"; // this has a type of string slice
        "Dummy".to_string() // returns a string instead of a struct
    }
}

async fn identity_handler(request: Request<Body>) -> Body {
    Body::from(request.uri().path().to_string())
}

#[tokio::test]
async fn test_hello_world() {
    let Html(s) = handler().await;

    assert!(s.contains("Hello, World!"));
}
