pub mod cmdtunnel {
    tonic::include_proto!("cmdtunnel");
}

use cmdtunnel::command_tunnel_client::CommandTunnelClient;
use cmdtunnel::{command_reply::Output, CommandRequest};

use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://[::1]:7786";
    let mut client = CommandTunnelClient::connect(addr).await?;
    let mut stream = client
        .run(Request::new(CommandRequest {
            command: "ctest -C Debug -R bsi_local_ut".into(),
        }))
        .await?
        .into_inner();

    while let Some(resp) = stream.message().await? {
        if let Some(output) = resp.output {
            match output {
                Output::Stdout(s) => println!("S: {}", s),
                Output::Stderr(s) => println!("E: {}", s),
            }
        }
    }

    Ok(())
}
