use actix_cors::Cors;
use actix_web::{
    guard,
    http::header::HOST,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use fluentci_core::deps::Graph;
use fluentci_ext::runner::Runner;
use fluentci_graphql::{schema::Query, FluentCISchema};
use owo_colors::OwoColorize;
use std::{
    env,
    sync::{mpsc, Arc, Mutex},
};

#[actix_web::post("/graphql")]
async fn index_graphql(schema: web::Data<FluentCISchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::get("/graphiql")]
async fn index_graphiql(req: HttpRequest) -> Result<HttpResponse> {
    let host = req
        .headers()
        .get(HOST)
        .unwrap()
        .to_str()
        .unwrap()
        .split(":")
        .next()
        .unwrap();

    let port = env::var("FLUENTCI_PORT").unwrap_or("6880".to_string());
    let graphql_endpoint = format!("http://{}:{}/graphql", host, port);
    let ws_endpoint = format!("ws://{}:{}/graphql", host, port);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint(&graphql_endpoint)
                .subscription_endpoint(&ws_endpoint)
                .finish(),
        ))
}

async fn index_ws(
    schema: web::Data<FluentCISchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

pub async fn start(listen: &str) -> std::io::Result<()> {
    let banner = r#"

    ________                 __  __________   ______            _          
   / ____/ /_  _____  ____  / /_/ ____/  _/  / ____/___  ____ _(_)___  ___ 
  / /_  / / / / / _ \/ __ \/ __/ /    / /   / __/ / __ \/ __ `/ / __ \/ _ \
 / __/ / / /_/ /  __/ / / / /_/ /____/ /   / /___/ / / / /_/ / / / / /  __/
/_/   /_/\__,_/\___/_/ /_/\__/\____/___/  /_____/_/ /_/\__, /_/_/ /_/\___/ 
                                                      /____/               

  "#;

    println!("{}", banner.bright_green());
    let port =
        env::var("FLUENTCI_ENGINE_PORT").unwrap_or(listen.split(":").last().unwrap().to_string());
    let host =
        env::var("FLUENTCI_ENGINE_HOST").unwrap_or(listen.split(":").next().unwrap().to_string());
    let addr = format!("{}:{}", host, port);
    println!(
        "Server is running on {}",
        format!("{}", addr).bright_green()
    );

    let (tx, rx) = mpsc::channel();

    let graph = Arc::new(Mutex::new(Graph::new(
        tx,
        Arc::new(Box::new(Runner::default())),
    )));

    let schema = Schema::build(
        Query::default(),
        EmptyMutation::default(),
        EmptySubscription::default(),
    )
    .data(graph)
    .data(Arc::new(Mutex::new(rx)))
    .finish();

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(Data::new(schema.clone()))
            .wrap(cors)
            .service(index_graphql)
            .service(index_graphiql)
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
    })
    .bind(addr)?
    .run()
    .await
}
