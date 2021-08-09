pub mod cmdtunnel {
    tonic::include_proto!("cmdtunnel");
}

use std::env;

use cmdtunnel::command_tunnel_client::CommandTunnelClient;
use cmdtunnel::{command_reply::Output, CommandRequest};

use console::style;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = env::args().nth(1).unwrap_or_default();
    let args = env::args().skip(2).collect::<Vec<_>>();

    let addr = env::var("CMD_TUNNEL_SERVER").unwrap_or_else(|_| "http://[::1]:7786".to_string());
    let mut client = CommandTunnelClient::connect(addr).await?;
    let mut stream = client
        .run(Request::new(CommandRequest { command, args }))
        .await?
        .into_inner();

    while let Some(output) = stream.message().await?.and_then(|resp| resp.output) {
        match output {
            Output::Stdout(s) => println!("{}", style(s).for_stdout()),
            Output::Stderr(s) => eprintln!("{}", style(s).green()),
        }
    }

    Ok(())
}
