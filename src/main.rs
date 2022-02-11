#![deny(warnings)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::process::Command;

async fn command(input: Request<Body>) -> String {
    let whole_body = hyper::body::to_bytes(input.into_body()).await.unwrap();
    let cmd = String::from_utf8(whole_body.iter().cloned().collect::<Vec<u8>>()).unwrap();
    let command_with_args: Vec<&str> = cmd.split_whitespace().collect();
    let output = Command::new(command_with_args[0])
        .args(&command_with_args[1..])
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

async fn ping(input: Request<Body>) -> String {
    let whole_body = hyper::body::to_bytes(input.into_body()).await.unwrap();
    let cmd = String::from_utf8(whole_body.iter().cloned().collect::<Vec<u8>>()).unwrap();
    let command_with_args: Vec<&str> = cmd.split_whitespace().collect();
    let output = Command::new("ping")
        .args(&command_with_args)
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

async fn curl(input: Request<Body>) -> String {
    let whole_body = hyper::body::to_bytes(input.into_body()).await.unwrap();
    let cmd = String::from_utf8(whole_body.iter().cloned().collect::<Vec<u8>>()).unwrap();
    let command_with_args: Vec<&str> = cmd.split_whitespace().collect();
    let output = Command::new("curl")
        .args(&command_with_args)
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

        (&Method::POST, "/") => Ok(Response::new(Body::from(command(req).await))),
        (&Method::POST, "/ping") => Ok(Response::new(Body::from(ping(req).await))),
        (&Method::POST, "/curl") => Ok(Response::new(Body::from(curl(req).await))),

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], 8080).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service)) });
    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
