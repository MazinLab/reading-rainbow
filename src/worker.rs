use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use gen3_rpc::{DDCChannelConfig, DSPScaleError};
use num::Complex;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::mpsc::{Receiver, Sender},
};

use tokio::runtime::Runtime;

// Define RPC commands for setting and getting the FFT scale
pub enum RPCCommand {
    SetFFTScale(u16),
    GetFFTScale,
}

// Define RPC responses for connection status and FFT scale
pub enum RPCResponse {
    Connected,
    FFTScale(Option<u16>),
}

pub fn worker_thread(
    command: Receiver<RPCCommand>,
    response: Sender<RPCResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        tokio::task::LocalSet::new()
            .run_until(async move {
                let stream = tokio::net::TcpStream::connect(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    54321,
                ))
                .await?;
                stream.set_nodelay(true)?;
                let (reader, writer) =
                    tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                let network = twoparty::VatNetwork::new(
                    futures::io::BufReader::new(reader),
                    futures::io::BufWriter::new(writer),
                    rpc_twoparty_capnp::Side::Client,
                    capnp::message::ReaderOptions {
                        traversal_limit_in_words: Some(usize::MAX),
                        nesting_limit: i32::MAX,
                    },
                );

                let mut rpc_system = RpcSystem::new(Box::new(network), None);
                let board = gen3_rpc::client::Gen3Board {
                    client: rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server),
                };

                // Send a connected response to the GUI
                response.send(RPCResponse::Connected).unwrap();

                tokio::task::spawn_local(rpc_system);

                // Get the DSP scale from the board
                let mut dsp_scale = board.get_dsp_scale().await?;
                loop {
                    match command.recv().unwrap() {
                        // Handle the SetFFTScale command
                        RPCCommand::SetFFTScale(i) => {
                            let r = dsp_scale.set_fft_scale(i).await;
                            match r {
                                Ok(i) => response.send(RPCResponse::FFTScale(Some(i))).unwrap(),
                                Err(DSPScaleError::Clamped(i)) => {
                                    response.send(RPCResponse::FFTScale(Some(i))).unwrap()
                                }
                                Err(_) => response.send(RPCResponse::FFTScale(None)).unwrap(),
                            }
                        }
                        // Handle the GetFFTScale command
                        RPCCommand::GetFFTScale => {
                            let r = dsp_scale.get_fft_scale().await;
                            response.send(RPCResponse::FFTScale(r.ok())).unwrap()
                        }
                    }
                }
            })
            .await
    })
}
