use std::io::{Read, Write};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    // パラメタを受け取る
    let local = get_required_env("LOCAL");
    let remote = get_required_env("REMOTE");

    // TCP ソケットを作成し、ローカル側のアドレスにバインドする
    let listener = TcpListener::bind(local, false).expect("TCP ソケットを作成できませんでした");

    // クライアントから新しい接続があるたびにストリームを返す
    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                // プロキシ先のサーバーに接続する
                let server = TcpStream::connect(&remote).expect("プロキシ先のサーバーに接続できませんでした");

                println!(
                    "{} <-> {} // {} <-> {}",
                    client.local_addr().unwrap(),
                    client.peer_addr().unwrap(),
                    server.local_addr().unwrap(),
                    server.peer_addr().unwrap()
                );

                proxy(client, server)?;
            }
            Err(e) => {
                eprintln!("クライアントからの接続を受け入れられませんでした: {}", e);
            }
        }
    }

    return Ok(());
}

fn get_required_env(key: &str) -> String {
    std::env::var(key).expect(&format!("{} が未設定です。", key))
}

fn proxy(mut client: TcpStream, mut server: TcpStream) -> std::io::Result<()> {
    // クライアントからリクエストを受け取る
    let mut request: Vec<u8> = Vec::new();
    loop {
        let mut buf = [0; 1024];
        let bytes_read = client.read(&mut buf).unwrap();
        request.extend_from_slice(&buf[..bytes_read]);

        if bytes_read < 1024 {
            break;
        }
    }

    println!("request :\n{}", std::str::from_utf8(&request).unwrap());

    // プロキシ先のサーバーにリクエストを送信する
    server.write(&request).unwrap();

    // プロキシ先のサーバーからレスポンスを受け取り、クライアントに送信する
    loop {
        let mut response = [0; 1024];
        let bytes_read = server.read(&mut response).unwrap();
        client.write(&response[..bytes_read]).unwrap();

        if bytes_read < 1024 {
            break;
        }
    }

    // ストリームを閉じる
    client.shutdown(Shutdown::Both).unwrap();
    server.shutdown(Shutdown::Both).unwrap();

    Ok(())
}
