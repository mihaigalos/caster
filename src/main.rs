#![deny(warnings)]

use autoclap::autoclap;
use clap::Command;
use clap::Arg;
use std::vec::Drain;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use chrono::prelude::*;
use hyper::server::conn::AddrStream;
use std::net::SocketAddr;
use std::process::Command as StdProcessCommand;

use colored::Colorize;
use tower::util::BoxService;

async fn execute<'a>(
    command: &str,
    args: Drain<'_, &'a str>,
    remote_address: SocketAddr,
) -> String {
    let args_string = args.as_slice().join(" ");
    let bash_args = [
        "-c".to_string(),
        command.to_string() + " " + &args.as_slice().join(" "),
    ];
    let output = StdProcessCommand::new("bash")
        .args(bash_args)
        .output()
        .expect("failed to execute command");
    let mut result = String::from_utf8(output.stdout).unwrap();

    let exit_status = output.status.code().unwrap();
    if exit_status != 0 {
        result = result
            + "Exit status: "
            + &exit_status.to_string()
            + "\n\n"
            + "stderr:\n"
            + &String::from_utf8(output.stderr).unwrap()
            + "\n";
    }
    let now: DateTime<Utc> = Utc::now();
    let pos_colon = remote_address.to_string().find(':').unwrap();

    let exit_status_string = match exit_status {
        0 => "OK".bright_green().bold(),
        _ => exit_status.to_string().bright_red().bold(),
    };
    println!(
        "{:width$} {:<15} {:>6} {:>8} {:<16}",
        now.to_string().bright_green().bold(),
        remote_address.to_string()[..pos_colon].bright_cyan().bold(),
        exit_status_string,
        command.bright_blue().bold(),
        args_string.yellow().bold(),
        width = 32
    );
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
    if explicit_command.is_empty() {
        command = command_with_args[0];
        args = command_with_args.drain(1..);
    } else {
        command = explicit_command;
        args = command_with_args.drain(0..);
    }

    return execute(command, args, remote_address).await;
}

async fn service(
    req: Request<Body>,
    remote_address: SocketAddr,
    is_secure: bool,
) -> Result<Response<Body>, hyper::Error> {
    if is_secure && (req.method() == Method::POST && req.uri().path() == ("/")) {
        let message = "Server started with --secure. Only explicit endpoints like /ping and /curl are available.\n";
        let mut response = Response::new(Body::from(message));
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        return Ok(response);
    }
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Example usage:\n  curl localhost:8080 -XPOST -d 'ls'\n",
        ))),

        (&Method::POST, "/") => Ok(Response::new(Body::from(
            command("", req, remote_address).await,
        ))),
        (&Method::POST, "/jq") => Ok(Response::new(Body::from(
            command("jq", req, remote_address).await,
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
            let svc = BoxService::new(service_fn(move |req| {
                service(req, remote_addressess, is_secure)
            }));
            Ok::<_, hyper::Error>(svc)
        }
    });

    let server = Server::bind(&addr).serve(service);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    println!(
        "{}\n",
        format!(
            "Caster {} - Listening on http://{}",
            env!("CARGO_PKG_VERSION"),
            addr
        )
        .yellow()
    );
    println!(
        "{:width$} {:<15} {:>6} {:>8} {:<16}",
        "Timestamp".yellow(),
        "IP".yellow(),
        "Status".yellow(),
        "Command".yellow(),
        "Args".yellow(),
        width = 33
    );
    if let Err(e) = graceful.await {
        eprintln!("server error: {e}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let app: clap::Command = autoclap!()
        .arg(
            Arg::new("secure")
                .long("secure")
                .short('s')
                .help("Only run explicit commands from endpoints such as /ping and /curl to avoid sensitive data leakage."),
        );

    let args = app.clone().try_get_matches().unwrap_or_else(|e| e.exit());
    run_server(args.get_flag("secure")).await;
    Ok(())
}
