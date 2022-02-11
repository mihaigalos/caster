#![deny(warnings)]

use autoclap::autoclap;
use clap::App;
use clap::Arg;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::process::Command;

async fn command(explicit_command: &str, input: Request<Body>) -> String {
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
    return result;
}

async fn service(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Example usage:\n  curl localhost:8080 -XPOST -d 'ls'\n",
        ))),

        (&Method::POST, "/") => Ok(Response::new(Body::from(command("", req).await))),
        (&Method::POST, "/ping") => Ok(Response::new(Body::from(command("ping", req).await))),
        (&Method::POST, "/curl") => Ok(Response::new(Body::from(command("curl", req).await))),

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn service_secure(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Example usage:\n  curl localhost:8080 -XPOST -d 'ls'\n",
        ))),

        (&Method::POST, "/ping") => Ok(Response::new(Body::from(command("ping", req).await))),
        (&Method::POST, "/curl") => Ok(Response::new(Body::from(command("curl", req).await))),

        _ => Ok(Response::new(Body::from(
            "Server started with --secure. Only explicit endpoints like /ping and /curl are available.\n",
        ))),
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
    let addr = ([0, 0, 0, 0], 8080).into();

    if args.is_present("secure") {
        let service =
            make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service_secure)) });
        let server = Server::bind(&addr).serve(service);
        server.await?;
    } else {
        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service)) });
        let server = Server::bind(&addr).serve(service);
        server.await?;
    }
    println!("Listening on http://{}", addr);
    Ok(())
}
