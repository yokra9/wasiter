use std::io::{Read, Write};
use wasmedge_wasi_socket::{nslookup, Shutdown, SocketAddr, TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    // パラメタを受け取る
    let local = get_required_env("LOCAL");
    let remote = get_required_env("REMOTE");

    let local =
        to_socket_address(&local).expect("パラメタをソケットアドレスに変換できませんでした");
    let remote =
        to_socket_address(&remote).expect("パラメタをソケットアドレスに変換できませんでした");

    // TCP ソケットを作成し、ローカル側のアドレスにバインドする
    let listener = TcpListener::bind(local, false).unwrap();

    // クライアントから新しい接続があるたびにストリームを返す
    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                // プロキシ先のサーバーに接続する
                let server = TcpStream::connect(&remote).unwrap();

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
    return std::env::var(key).unwrap_or_else(|err| {
        panic!("{}が未設定です: {}", key, err);
    });
}

fn to_socket_address(str: &str) -> Result<SocketAddr, &'static str> {
    let mut iter = str.split(":");
    let host = iter.next().expect("パラメタの書式が間違っています");
    let port = iter.next().expect("パラメタの書式が間違っています");

    let addr = nslookup(host, port).expect("名前解決に失敗しました");

    if addr.len() == 0 {
        return Err("名前解決に失敗しました");
    }

    Ok(addr[0])
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
