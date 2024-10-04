use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::error::Error;
use tokio::time::{sleep, Duration};
use clap::Parser;
use serde::{Serialize, Deserialize};
use mysql::{self, prelude::Queryable, params,OptsBuilder};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The user name for the RPC server
    #[arg(long, required = true)]
    rpc_user: String,
    /// The password for the RPC server
    #[arg(long, required = true)]
    rpc_pwd: String,
    /// The RPC server URL
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,
    /// The db user name
    #[arg(long, required = true)]
    db_user: String,
    /// The db password
    #[arg(long, required = true)]
    db_pwd: String,
    /// The db host
    #[arg(long, default_value = "localhost" )]
    db_host: String,
}

// TBD
// #[derive(Debug, Serialize, Deserialize)]
// pub struct Transaction {
//     pub tx_hash: String,
//     pub tx_amount: f64,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub id: usize,
    pub hash: String,
    pub height: usize,
    pub timestamp: usize,
    pub size: usize,
    pub merkle_root: String,
    pub num_transactions: usize,      
    // pub transactions: Vec<Transaction>,
}

fn best_ten() -> Result<(), Box<dyn Error>> {
    
    let args = Args::parse();
    let rpc_url = &args.rpc_url;
    let auth = Auth::UserPass(args.rpc_user.to_string(), args.rpc_pwd.to_string());
    println!("Connecting to Bitcoin Core RPC server at {}", rpc_url);
    println!("Authenticating as user {}", args.rpc_user);
    let client = Client::new(rpc_url, auth)?;


    let best_block_hash = client.get_best_block_hash()?;


    let mut block_hash = best_block_hash;
    let mut block_dtos: Vec<BlockInfo> = Vec::new();
    for i in 1..=10 {

        let block_info = client.get_block_info(&block_hash)?;

        println!("Block {}: Height = {}", i, block_info.height);


        let block_dto: BlockInfo = BlockInfo {
            id: i as usize,
            hash: block_info.hash.to_string(),
            height: block_info.height,
            timestamp: block_info.time,
            size: block_info.size,
            merkle_root: block_info.merkleroot.to_string(),
            num_transactions: block_info.tx.len(),
        };
        // println!("Block Info: {:?}", block_dto);
        block_dtos.push(block_dto);

        match block_info.previousblockhash {
            Some(prev_hash) => block_hash = prev_hash,
            None => {
                println!("No more previous blocks.");
                break;
            }
        }
    }
    let opts = OptsBuilder::new()
    .user(Some(args.db_user))
    .pass(Some(args.db_pwd))
    .ip_or_hostname(Some(args.db_host))
    .tcp_port(3306)
    .db_name(Some("bitcoin_explorer"));

    let pool = mysql::Pool::new(opts)?;
    let mut conn = pool.get_conn()?;
    match  conn.exec_batch(
        r#"
        INSERT INTO Best_blocks (id, hash, height, timestamp, size, merkle_root, num_transactions)
        VALUES (:id, :hash, :height, :timestamp, :size, :merkle_root, :num_transactions)
        ON DUPLICATE KEY UPDATE
            hash = VALUES(hash),
            height = VALUES(height),
            timestamp = VALUES(timestamp),
            size = VALUES(size),
            merkle_root = VALUES(merkle_root),
            num_transactions = VALUES(num_transactions)
        "#,
        block_dtos.iter().map(|b_dto| {
            params! {
                "id" => b_dto.id,
                "hash" => &b_dto.hash,
                "height" => b_dto.height,
                "timestamp" => b_dto.timestamp,
                "size" => b_dto.size,
                "merkle_root" => &b_dto.merkle_root,
                "num_transactions" => b_dto.num_transactions,
            }
        })
    ) {
        Ok(_) => println!("Successfully inserted block info into database."),
        Err(e) => println!("Error inserting block info into database: {}", e),
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
