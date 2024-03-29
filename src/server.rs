use std::{
    io::{stdout, Write},
    process::Stdio,
};

pub mod cmdtunnel {
    tonic::include_proto!("cmdtunnel");
}

use clap::Parser;
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
        print!(
            "Running command: {}, args: {:?}...",
            req.get_ref().command,
            req.get_ref().args
        );
        stdout().lock().flush()?;

        let mut cmd = Command::new(req.get_ref().command.clone())
            .args(&req.get_ref().args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = cmd.stdout.take().map(BufReader::new);
        let stderr = cmd.stderr.take().map(BufReader::new);
        let stdout_lines = stdout.map(|s| s.lines());
        let stderr_lines = stderr.map(|s| s.lines());
        let (tx, rx) = mpsc::channel(1024);
        let (tx_done, mut rx_done) = mpsc::channel(1024);

        if let Some(mut stdout_lines) = stdout_lines {
            let tx = tx.clone();
            let tx_done = tx_done.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_lines.next_line().await {
                    if tx
                        .send(Ok(CommandReply {
                            output: Some(Output::Stdout(line)),
                        }))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                let _ = tx_done.send(()).await;
            });
        }

        if let Some(mut stderr_lines) = stderr_lines {
            let tx = tx.clone();
            let tx_done = tx_done.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    if tx
                        .send(Ok(CommandReply {
                            output: Some(Output::Stderr(line)),
                        }))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                let _ = tx_done.send(()).await;
            });
        }

        tokio::spawn(async move {
            let _ = rx_done.recv().await;
            println!("done.");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Parser, Debug)]
struct Opt {
    /// Host name or IP to listen on.
    #[clap(long, short('n'), default_value = "0.0.0.0")]
    host_name: String,

    /// Port to listen on.
    #[clap(long, short, default_value = "7786")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();
    let addr = format!("{}:{}", opt.host_name, opt.port).parse()?;
    let svr = CommandTunnelServer::new(CommandTunnelService);

    println!(
        "Listening for commands at http://{}:{}",
        opt.host_name, opt.port
    );
    Server::builder().add_service(svr).serve(addr).await?;

    Ok(())
}
