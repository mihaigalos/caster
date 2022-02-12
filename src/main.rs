#![deny(warnings)]

use autoclap::autoclap;
use clap::App;
use clap::Arg;
use std::vec::Drain;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use chrono::prelude::*;
use hyper::server::conn::AddrStream;
use std::net::SocketAddr;
use std::process::Command;

use tower::util::BoxService;

async fn execute<'a>(command: &str, args: Drain<'_, &'a str>) -> String {
    let output = Command::new(command)
        .args(args)
        .output()
        .expect("failed to execute command");
    let mut result = String::from_utf8(output.stdout).unwrap();

    if output.status.code().unwrap() != 0 {
        result = result
            + "Exit status: "
            + &output.status.code().unwrap().to_string()
            + "\n\n"
            + "stderr:\n"
            + &String::from_utf8(output.stderr).unwrap()
            + "\n";
    }
    result
}

async fn command(
    explicit_command: &str,
    input: Request<Body>,
    remote_address: SocketAddr,
) -> String {
    let whole_body = hyper::body::to_bytes(input.into_body()).await.unwrap();
    let cmd = String::from_utf8(whole_body.iter().cloned().collect::<Vec<u8>>()).unwrap();
    let mut command_with_args: Vec<&str> = cmd.split_whitespace().collect();

    let command;
    let args;
    if explicit_command == "" {
        command = command_with_args[0];
        args = command_with_args.drain(1..);
    } else {
        command = explicit_command;
        args = command_with_args.drain(0..);
    }

    let now: DateTime<Utc> = Utc::now();
    println!(
        "{} {} # {} {}",
        now,
        remote_address,
        command,
        args.as_slice().join(" ")
    );
    return execute(command, args).await;
}

async fn service(
    req: Request<Body>,
    remote_address: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Example usage:\n  curl localhost:8080 -XPOST -d 'ls'\n",
        ))),

        (&Method::POST, "/") => Ok(Response::new(Body::from(
            command("", req, remote_address).await,
        ))),
        (&Method::POST, "/ping") => Ok(Response::new(Body::from(
            command("ping", req, remote_address).await,
        ))),
        (&Method::POST, "/curl") => Ok(Response::new(Body::from(
            command("curl", req, remote_address).await,
        ))),

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn service_secure(
    req: Request<Body>,
    remote_address: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Example usage:\n  curl localhost:8080 -XPOST -d 'ls'\n",
        ))),

        (&Method::POST, "/ping") => Ok(Response::new(Body::from(command("ping", req, remote_address).await))),
        (&Method::POST, "/curl") => Ok(Response::new(Body::from(command("curl", req, remote_address).await))),

        _ => Ok(Response::new(Body::from(
            "Server started with --secure. Only explicit endpoints like /ping and /curl are available.\n",
        ))),
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn run_server(is_secure: bool) {
    let addr = ([0, 0, 0, 0], 8080).into();

    let service = make_service_fn(move |conn: &AddrStream| {
        let remote_addressess = conn.remote_addr();
        async move {
            let svc = if is_secure {
                BoxService::new(service_fn(move |req| {
                    service_secure(req, remote_addressess)
                }))
            } else {
                BoxService::new(service_fn(move |req| service(req, remote_addressess)))
            };
            Ok::<_, hyper::Error>(svc)
        }
    });

    let server = Server::bind(&addr).serve(service);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    println!(
        "{}",
        format!(
            "Caster {} - Listening on http://{}",
            env!("CARGO_PKG_VERSION"),
            addr
        )
    );
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app: clap::App = autoclap!();
    let args = app
        .arg(
            Arg::new("secure")
                .long("secure")
                .short('s')
                .help("Only run explicit commands from endpoints such as /ping and /curl to avoid sensitive data leakage."),
        )
        .try_get_matches()
        .unwrap_or_else(|e| e.exit());

    run_server(args.is_present("secure")).await;
    Ok(())
}
