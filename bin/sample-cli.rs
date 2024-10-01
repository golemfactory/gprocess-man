use anyhow::{bail, Result};
use gprocess_proto::gprocess::api::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use prost::Message;

async fn write_request(request: &Request, stream: &mut TcpStream) -> Result<()> {
    stream.write_u32(request.encoded_len() as u32).await?;
    stream.write_all(&request.encode_to_vec()).await?;

    Ok(())
}

async fn read_response(stream: &mut TcpStream) -> Result<Response> {
    let length = stream.read_u32().await?;
    let mut buf = vec![0; length as usize];
    stream.read_exact(&mut buf).await?;

    let response = Response::decode(buf.as_slice());
    match response {
        Ok(response) => Ok(response),
        Err(e) => bail!(e),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = tokio::net::TcpStream::connect("localhost:1234").await?;

    let mut request_id = 0u32;

    let start_request = StartRequest {
        program: "/bin/cat".to_string(),
        // args: vec!["10".into(), "500".into()],
        stdin: Some(Stream::Pipe as i32),
        stdout: Some(Stream::Pipe as i32),
        stderr: Some(Stream::Pipe as i32),
        ..Default::default()
    };
    // let start_request = StartRequest {
    //     program: "mc".to_string(),
    //     // args: vec!["10".into(), "500".into()],
    //     stdin: Some(Stream::Pipe as i32),
    //     stdout: Some(Stream::Pipe as i32),
    //     stderr: Some(Stream::Pipe as i32),
    //     ..Default::default()
    // };
    // start_request.program = "mc".to_string();

    let request = Request {
        request_id,
        command: Some(request::Command::Start(start_request)),
    };

    write_request(&request, &mut stream).await?;
    let response = read_response(&mut stream).await?;

    let start_response: response::Command = response.command.unwrap();
    let start_response = match start_response {
        response::Command::Start(start_response) => start_response,
        _ => panic!("Invalid response"),
    };
    let stdout = start_response.stdout.unwrap();

    loop {
        // println!("Requesting read");
        let read_request = ReadRequest {
            pid: start_response.pid,
            len: 1024,
            stream: stdout,
        };

        request_id += 1;

        let request = Request {
            request_id,
            command: Some(request::Command::Read(read_request)),
        };

        // println!("request_id: {:?}", request.request_id.clone());

        write_request(&request, &mut stream).await?;
        let response = read_response(&mut stream).await?;
        let command = response.command.clone().expect("Failed to get command");

        match command {
            response::Command::Read(read_response) => {
                // println!(" {:?}", response);
                if read_response.data.len() == 0 {
                    println!("<EOS>");
                    break;
                }
                print!("{}", String::from_utf8_lossy(&read_response.data));
            }
            response::Command::Error(error) => {
                println!(" Invalid response: {:?}", error);
                break;
            }
            _ => {
                println!(" Invalid response: {:?}", response);
            }
        }
    }

    stream.shutdown().await?;

    Ok(())
}
