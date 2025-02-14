mod gui;
mod logger;
mod status;
mod sweep;

use tokio::runtime::Runtime;

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    thread,
};

fn worker_thread() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?; // Initialize tokio runtime
    // Start running the tokio runtime
    rt.block_on(async {
        tokio::task::LocalSet::new()
            .run_until(async move {
                // Connect to TCP (nodelay(true) reduces latency for small packets )
                let stream = tokio::net::TcpStream::connect(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    54321,
                ))
                .await?;
                stream.set_nodelay(true)?;
                // Split the stream into reader and writer
                let (reader, writer) =
                    tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();

                // Initialize RPC System (computer to computer connection)
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

                tokio::task::spawn_local(rpc_system);

                // Initialize DAC and DSP interaction 
                let mut dactable = board.get_dac_table().await?;
                let mut dsp_scale = board.get_dsp_scale().await?;

                let mut d = dactable.get_dac_table().await?; // Retrieve first 16 DAC table values and print 
                println!("Before: {:?}", d[..16].iter().collect::<Vec<_>>());

                // Update first 3 DAC table values and print 
                d[0].re = 8;
                d[1].im = 32;
                d[2].re = 0x55;
                dactable.set_dac_table(d).await?;

                let p = dactable.get_dac_table().await?;
                println!("After: {:?}", p[..16].iter().collect::<Vec<_>>());

                let scale = dsp_scale.get_fft_scale().await?; // Retrieve current DSP scale and print 
                println!("Starting Scale: {:?}", scale);

                // Set new DSP scale and print 
                let scale = dsp_scale.set_fft_scale(0xF0F).await;
                println!("Set Valid Scale: {:?}", scale);

                let scale = dsp_scale.set_fft_scale(0xF0F0).await;
                println!("Set Invalid Scale: {:?}", scale);
                
                // Return from async block 
                Ok(())
            })
            .await
    })
}

fn main() {
    let worker = thread::spawn(move || {
        worker_thread().unwrap();
    });

    gui::run_gui();

    worker.join().unwrap();
}
