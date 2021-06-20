use std::process::Stdio;

pub mod cmdtunnel {
    tonic::include_proto!("cmdtunnel");
}

use cmdtunnel::{
    command_reply::Output,
    command_tunnel_server::{CommandTunnel, CommandTunnelServer},
    CommandReply, CommandRequest,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
struct CommandTunnelService;

#[tonic::async_trait]
impl CommandTunnel for CommandTunnelService {
    type RunStream = ReceiverStream<Result<CommandReply, Status>>;

    async fn run(&self, req: Request<CommandRequest>) -> Result<Response<Self::RunStream>, Status> {
        println!(
            "Running command: {}, args: {:?}",
            req.get_ref().command,
            req.get_ref().args
        );

        let mut cmd = Command::new(req.get_ref().command.clone())
            .args(&req.get_ref().args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = cmd.stdout.take().map(|stdout| BufReader::new(stdout));
        let stderr = cmd.stderr.take().map(|stderr| BufReader::new(stderr));
        let stdout_lines = stdout.map(|s| s.lines());
        let stderr_lines = stderr.map(|s| s.lines());
        let (tx, rx) = mpsc::channel(1024);

        if let Some(mut stdout_lines) = stdout_lines {
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(line) = stdout_lines.next_line().await.unwrap() {
                    tx.send(Ok(CommandReply {
                        output: Some(Output::Stdout(line)),
                    }))
                    .await
                    .unwrap();
                }
            });
        }

        if let Some(mut stderr_lines) = stderr_lines {
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(line) = stderr_lines.next_line().await.unwrap() {
                    tx.send(Ok(CommandReply {
                        output: Some(Output::Stderr(line)),
                    }))
                    .await
                    .unwrap();
                }
            });
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:7786".parse().unwrap();
    let svr = CommandTunnelServer::new(CommandTunnelService);

    println!("Listening for commands at http://localhost:7786");
    Server::builder().add_service(svr).serve(addr).await?;

    Ok(())
}
