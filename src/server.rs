pub mod cmdtunnel {
    tonic::include_proto!("cmdtunnel");
}

use cmdtunnel::{
    command_reply::Output,
    command_tunnel_server::{CommandTunnel, CommandTunnelServer},
    CommandReply, CommandRequest,
};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
struct CommandTunnelService;

#[tonic::async_trait]
impl CommandTunnel for CommandTunnelService {
    type RunStream = ReceiverStream<Result<CommandReply, Status>>;

    async fn run(&self, req: Request<CommandRequest>) -> Result<Response<Self::RunStream>, Status> {
        println!("Run command: {}", req.get_ref().command);

        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            for i in 0..10 {
                sleep(Duration::from_millis(500)).await;
                tx.send(Ok(CommandReply {
                    output: Some(Output::Stdout(format!("{} boo", i + 1).into())),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:7786".parse().unwrap();
    let svr = CommandTunnelServer::new(CommandTunnelService);

    Server::builder().add_service(svr).serve(addr).await?;

    Ok(())
}
