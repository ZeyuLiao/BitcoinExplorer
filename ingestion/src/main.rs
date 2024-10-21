use bitcoincore_rpc::{
    json::{HashOrHeight, TxOutSetHashType},
    Auth, Client, RpcApi,
};
use clap::Parser;
use mysql::{self, params, prelude::Queryable, OptsBuilder, Pool};
use redis::{Client as RedisClient, Commands};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::time::{sleep, Duration};

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
    #[arg(long, default_value = "localhost")]
    db_host: String,
    /// The redis host
    #[arg(long, default_value = "localhost")]
    redis_host: String,
    /// The redis port
    #[arg(long, default_value = "6379")]
    redis_port: u16,
    /// The redis password
    #[arg(long, default_value = "")]
    redis_pwd: String,
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

// #[derive(Debug)]
// struct BlockchainMetrics {
//     block_height: u64,
//     blockchain_size: u64,
//     network_hashrate: f64,
//     difficulty: f64,
//     mempool_size: usize,
//     total_supply: f64,
// }

fn initialize_connection() -> Result<Pool, Box<dyn Error>> {
    let args = Args::parse();
    let opts = OptsBuilder::new()
        .user(Some(args.db_user))
        .pass(Some(args.db_pwd))
        .ip_or_hostname(Some(args.db_host))
        .tcp_port(3306)
        .db_name(Some("bitcoin_explorer"));

    let pool = mysql::Pool::new(opts)?;
    Ok(pool)
}

fn fetch_metrics(
    bitcoin_client: &Client,
    redis_client: &RedisClient,
) -> Result<(), Box<dyn Error>> {
    println!("######METRICS######");
    let block_height = bitcoin_client.get_block_count()?;

    let blockchain_info = bitcoin_client.get_blockchain_info()?;
    let blockchain_size = blockchain_info.size_on_disk;

    let network_hashrate = bitcoin_client.get_network_hash_ps(None, None)? as f64;

    let difficulty = bitcoin_client.get_difficulty()?;

    let mempool_info = bitcoin_client.get_mempool_info()?;
    let mempool_size = mempool_info.size;

    let tx_out_set_info = bitcoin_client.get_tx_out_set_info(
        Some(TxOutSetHashType::Muhash),
        Some(HashOrHeight::Height(block_height)),
        Some(true),
    )?;
    let total_supply = tx_out_set_info.total_amount.to_btc();

    let mut redis_con = redis_client.get_connection()?;
    let _: () = redis_con.mset(&[
        ("block_height", block_height.to_string()),
        ("blockchain_size", blockchain_size.to_string()),
        ("network_hashrate", network_hashrate.to_string()),
        ("difficulty", difficulty.to_string()),
        ("mempool_size", mempool_size.to_string()),
        ("total_supply", total_supply.to_string()),
    ])?;

    println!("Successfully updated Redis with metrics.");

    Ok(())
}

fn best_ten(client: &Client, pool: &Pool) -> Result<(), Box<dyn Error>> {
    println!("######BEST10######");
    let best_block_hash = client.get_best_block_hash()?;

    let mut block_hash = best_block_hash;
    let mut block_dtos: Vec<BlockInfo> = Vec::new();
    for i in 1..=10 {
        let block_info = client.get_block_info(&block_hash)?;

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

    // Update to mysql
    let mut conn = pool.get_conn()?;
    match conn.exec_batch(
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
        }),
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
        println!("-------------------------------------------------------------");
        println!("Time: {:?}", _now);

        let args = Args::parse();

        // Connect to Bitcoin Core RPC server
        let rpc_url = &args.rpc_url;
        let auth = Auth::UserPass(args.rpc_user.to_string(), args.rpc_pwd.to_string());
        println!("Connecting to Bitcoin Core RPC server at {}", rpc_url);
        println!("Authenticating as user {}", args.rpc_user);
        let _bitcoin_client = match Client::new(rpc_url, auth) {
            Ok(client) => client,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        // Connect to MySQL database
        let _pool = match initialize_connection() {
            Ok(pool) => pool,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };
        // Connect to Redis
        let _redis_client = match redis::Client::open(format!(
            "redis://default:{}@{}:{}",
            args.redis_pwd, args.redis_host, args.redis_port
        ))
        .map_err(|e| format!("Error connecting to Redis: {}", e))
        {
            Ok(client) => client,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        // Fetch metrics from Bitcoin Core RPC server and update Redis
        match fetch_metrics(&_bitcoin_client, &_redis_client) {
            Ok(_) => {}
            Err(e) => println!("Error fetching metrics from Bitcoin Core RPC server: {}", e),
        };
        // Fetch best 10 blocks from Bitcoin Core RPC server and update MySQL database
        match best_ten(&_bitcoin_client, &_pool) {
            Ok(_) => {}
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
