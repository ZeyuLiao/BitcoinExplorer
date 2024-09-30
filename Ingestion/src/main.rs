use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::error::Error;
use tokio::time::{sleep, Duration};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The user name for the RPC server
    #[arg(long, required = true)]
    user: String,
    /// The password for the RPC server
    #[arg(long, required = true)]
    pwd: String,
}

fn best_ten() -> Result<(), Box<dyn Error>> {
    
    let args = Args::parse();
    let rpc_url = "http://127.0.0.1:8332";
    let auth = Auth::UserPass(args.user.to_string(), args.pwd.to_string());
    println!("Connecting to Bitcoin Core RPC server at {}", rpc_url);
    println!("Authenticating as user {}", args.user);
    let client = Client::new(rpc_url, auth)?;


    let best_block_hash = client.get_best_block_hash()?;


    let mut block_hash = best_block_hash;
    for i in 1..=10 {

        let block_info = client.get_block_info(&block_hash)?;

        println!("Block {}: Height = {}", i, block_info.height);


        match block_info.previousblockhash {
            Some(prev_hash) => block_hash = prev_hash,
            None => {
                println!("No more previous blocks.");
                break;
            }
        }
    }

    Ok(())
}

async fn run_periodic_task() {
    loop {
        //print time stamp
        let _now = chrono::Utc::now();
        println!("Time: {:?}", _now);
        let _err = best_ten();
        match _err {
            Ok(_) => println!("Success"),
            Err(e) => println!("Error: {}", e),
        }
        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_periodic_task().await;
    Ok(())
}
