use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use gen3_rpc::{client::ExclusiveDroppableReference, Attens, DSPScaleError, Hertz};
use gen3_rpc::DDCChannelConfig;
use num::Complex;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::mpsc::{Receiver, Sender},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::runtime::Runtime;

// Define RPC commands for setting and getting the FFT scale, DAC table, and IF board
pub enum RPCCommand {
    SetFFTScale(u16),
    GetFFTScale,
    GetDACTable,
    SetDACTable(Box<[Complex<i16>; 524288]>),
    GetIFFreq,
    SetIFFreq(Hertz),
    GetIFAttens,
    SetIFAttens(Attens),
}

// Define RPC responses for connection status, FFT scale, DAC table, and IF board
pub enum RPCResponse {
    Connected(SystemTime), // Include timestamp in the Connected response
    FFTScale(Option<u16>),
    DACTable(Option<Box<[Complex<i16>; 524288]>>),
    IFFreq(Option<Hertz>),
    IFAttens(Option<Attens>),
}

pub fn worker_thread(
    command: Receiver<RPCCommand>,
    response: Sender<RPCResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        tokio::task::LocalSet::new()
            .run_until(async move {
                println!("Attempting to connect to server at 128.111.23.124:4242");
                let stream = tokio::net::TcpStream::connect(SocketAddrV4::new(
                    Ipv4Addr::new(128, 111, 23, 124), // Ensure this matches the server's IP address
                    4242, // Ensure this matches the server's port
                ))
                .await?;
                println!("Successfully connected to server");
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

                // RPC System initializes communication between us and the board 
                let mut rpc_system = RpcSystem::new(Box::new(network), None);

                let board = gen3_rpc::client::Gen3Board {
                    client: rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server),
                };

                // Send a connected response to the GUI with the current timestamp
                let start_time = SystemTime::now();
                response.send(RPCResponse::Connected(start_time)).unwrap();

                tokio::task::spawn_local(rpc_system);

                // Get DSP Scale, DAC Table, IF Board from board
                let mut dsp_scale: ExclusiveDroppableReference<_, _> = board.get_dsp_scale().await?.try_into_mut().await?.unwrap_or_else(|_| todo!());
                let mut dac_table = board.get_dac_table().await?.try_into_mut().await?.unwrap_or_else(|_| todo!());
                let mut if_board = board.get_if_board().await?.try_into_mut().await?.unwrap_or_else(|_| todo!());

                loop {
                    match command.recv().unwrap() {
                        // Handle the SetFFTScale command
                        RPCCommand::SetFFTScale(i) => {
                            println!("Received SetFFTScale command with value: {}", i);
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
                        // Handle the GetDACTable command
                        RPCCommand::GetDACTable => {
                            let r = dac_table.get_dac_table().await;
                            match r {
                                Ok(d) => response.send(RPCResponse::DACTable(Some(d))).unwrap(),
                                Err(e) => {
                                    eprintln!("Failed to get DAC table: {}", e);
                                    response.send(RPCResponse::DACTable(None)).unwrap()
                                },
                            }
                        }
                        // Handle the SetDACTable command
                        RPCCommand::SetDACTable(data) => {
                            let data_clone = data.clone();
                            let r = dac_table.set_dac_table(data).await;
                            match r {
                                Ok(_) => response.send(RPCResponse::DACTable(Some(data_clone))).unwrap(),
                                Err(e) => {
                                    eprintln!("Failed to set DAC table: {}", e);
                                    response.send(RPCResponse::DACTable(None)).unwrap()
                                },
                            }
                        }
                        // Handle the GetIFFreq command
                        RPCCommand::GetIFFreq => {
                            let r = if_board.get_freq().await;
                            response.send(RPCResponse::IFFreq(r.ok())).unwrap()
                        }
                        // Handle the SetIFFreq command
                        RPCCommand::SetIFFreq(freq) => {
                            let r = if_board.set_freq(freq).await;
                            match r {
                                Ok(f) => response.send(RPCResponse::IFFreq(Some(f))).unwrap(),
                                Err(e) => {
                                    eprintln!("Failed to set IF frequency: {:?}", e);
                                    response.send(RPCResponse::IFFreq(None)).unwrap()
                                },
                            }
                        }
                        // Handle the GetIFAttens command
                        RPCCommand::GetIFAttens => {
                            println!("Received GetIFAttens command");
                            let r = if_board.get_attens().await;
                            match r {
                                Ok(a) => response.send(RPCResponse::IFAttens(Some(a))).unwrap(),
                                Err(e) => {
                                    eprintln!("Failed to get IF attenuations: {:?}", e);
                                    response.send(RPCResponse::IFAttens(None)).unwrap()
                                },
                            }
                        }
                        // Handle the SetIFAttens command
                        RPCCommand::SetIFAttens(attens) => {
                            let r = if_board.set_attens(attens).await;
                            match r {
                                Ok(a) => response.send(RPCResponse::IFAttens(Some(a))).unwrap(),
                                Err(e) => {
                                    eprintln!("Failed to set IF attenuations: {:?}", e);
                                    response.send(RPCResponse::IFAttens(None)).unwrap()
                                },
                            }
                        }
                    }
                }
            })
            .await
    })
}
