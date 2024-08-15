use std::{io::{self, stdin, BufRead, BufReader}, sync::Arc};

use anyhow::Result;
use automerge::{transaction::Transactable, Automerge, ReadDoc, ROOT};
use clap::Parser;
use iroh::{net::key, node::{Node, ProtocolHandler}};

use protocol::IrohAutomergeProtocol;
use serde::de::value;
use tokio::sync::mpsc;
use tokio::io::{self as tokio_io, AsyncBufReadExt, BufReader as TokioBufReader};

mod protocol;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(long)]
    remote_id: Option<iroh::net::NodeId>,
}

///use tokio runtime and 非同期処理を行うためのアトリビュート
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opts = Cli::parse();

    //チャネルのセットアップとプロトコルの初期化
    let (sync_sender, mut sync_finished) = mpsc::channel(10);
    let automerge = IrohAutomergeProtocol::new(Automerge::new(), sync_sender);
    let iroh = Node::memory()
        .disable_docs()
        .build()
        .await?
        .accept(
            IrohAutomergeProtocol::ALPN,
            Arc::clone(&automerge) as Arc<dyn ProtocolHandler>,
        )
        .spawn()
        .await?;

    //nodeアドレス取得と表示
    let addr = iroh.node_addr().await?;
    println!("Running\nNode Id: {}", addr.node_id,);

    //プロバイダーモードとレシーバーモードの分岐
    if let Some(remote_id) = opts.remote_id {
        // on the provider side:
        let stdin = tokio_io::stdin();
        let reader = TokioBufReader::new(stdin);
        let mut lines = reader.lines();

        loop {
            println!("Enter a key and value separated by a space (or 'exit' to quit): ");
            if let Ok(Some(line)) = lines.next_line().await {
                println!("User input received: {}", line);
                let mut split = line.trim().splitn(2, ' ');
                let key = split.next();
                let value = split.next();

                if let (Some(key), Some(value)) = (key, value) {
                    println!("Sending data: key = {}, value = {}", key, value);
                    // Put some data in the document to sync
                    let mut doc = automerge.fork_doc().await;
                    let mut t = doc.transaction();
                    t.put(ROOT, key, value)?;
                    t.commit();
                    automerge.merge_doc(&mut doc).await?;
                    println!("Data committed and merged.");

                    // connect to the other node
                    let node_addr = iroh::net::NodeAddr::new(remote_id);
                    let retry_count = 0;
                    loop {
                        println!("Retry count: {}", retry_count);
                        match iroh.endpoint().connect(node_addr.clone(), IrohAutomergeProtocol::ALPN).await {
                            Ok(conn) => {
                                println!("Connected successfully.");
                                if let Err(e) = automerge.clone().initiate_sync(conn).await {
                                    println!("Sync error: {:?}", e);
                                    if e.to_string().contains("closed by peer") {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                    } else {
                                        break;
                                    }
                                } else {
                                    println!("Sync completed successfully");
                                    break;
                                }
                            },
                            Err(e) => {
                                println!("Connection error: {:?}", e);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    }
                } else if key == Some("exit") {
                    break;
                }else {
                    println!("Invalid input, please enter a key and value.");
                }
            }
        }
    } else {
        // on the receiver side:
        // --remote-idが指定されていない場合、レシーバーとして動作し、同期が完了するまで待機する
        loop {
            if let Some(doc) = sync_finished.recv().await {
                println!("Document received. Processing state...");
                let keys: Vec<_> = doc.keys(automerge::ROOT).collect();
                for key in keys {
                    let (value, _) = doc.get(automerge::ROOT, &key)?.unwrap();
                    println!("{} => {}", key, value);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                println!("Document processed.");
            } else {
                println!("Cannel closed, no more documents to receive.");
                break;
            }
        }
    }

    // finally shut down
    iroh.shutdown().await?;

    Ok(())
}
