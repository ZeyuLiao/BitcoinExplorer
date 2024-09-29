use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 设置连接到 Bitcoin Core 节点的 RPC 客户端
    let rpc_url = "http://43.153.58.6:8332";  // Bitcoin Core 默认端口
    let rpc_user = "bot";     // 你的 RPC 用户名
    let rpc_password = "iambot"; // 你的 RPC 密码
    let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());

    // 创建 RPC 客户端
    let client = Client::new(rpc_url, auth)?;

    // 获取最新区块的哈希
    let best_block_hash = client.get_best_block_hash()?;

    // 打印最近10个区块的高度
    let mut block_hash = best_block_hash;
    for i in 1..=10 {
        // 获取区块信息
        let block_info = client.get_block_info(&block_hash)?;

        // 打印区块高度
        println!("Block {}: Height = {}", i, block_info.height);

        // 获取前一个区块的哈希
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
