use tokio::net::{TcpListener, TcpStream};

// WasmEdge はマルチスレッドをサポートしていない
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    // パラメタを受け取る
    let local = get_required_env("LOCAL");
    let remote = get_required_env("REMOTE");

    // TCP ソケットを作成し、ローカル側のアドレスにバインドする
    let listener = TcpListener::bind(&local)
        .await
        .expect("TCP ソケットを作成できませんでした");

    loop {
        // クライアントからの接続を受け入れる
        let mut client = match listener.accept().await {
            Ok((c, _)) => c,
            Err(e) => {
                eprintln!("クライアントからの接続を受け入れられませんでした: {}", e);
                continue;
            }
        };

        // リモートサーバーに接続する
        let mut server = match TcpStream::connect(&remote).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("リモートサーバーに接続できませんでした: {}", e);
                continue;
            }
        };

        println!(
            "{} <-> {} // {} <-> {}",
            client.local_addr().unwrap(),
            client.peer_addr().unwrap(),
            server.local_addr().unwrap(),
            server.peer_addr().unwrap()
        );

        // それぞれのストリームから読み取ったデータをリアルタイムで反対側のストリームに書き込む
        let (c2s, s2c) = match tokio::io::copy_bidirectional(&mut client, &mut server).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("IO エラーが発生しました: {}", e);
                break;
            }
        };

        println!(
            "{} bytes from client to server, {} bytes from server to client\n",
            c2s, s2c
        );
    }

    return Ok(());
}

fn get_required_env(key: &str) -> String {
    std::env::var(key).expect(&format!("{} が未設定です。", key))
}
