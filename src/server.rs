use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};

type Writer = tokio::io::WriteHalf<BufStream<TcpStream>>;
type Reader = tokio::io::ReadHalf<BufStream<TcpStream>>;

pub async fn listen(port: u16) -> anyhow::Result<()> {
    let address = format!("0.0.0.0:#{}", port);
    let listener = TcpListener::bind(address).await?;

    println!("Listening on port: {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            match handle_connection(socket).await {
                Ok(_) => println!("Connection closed: {}", addr),
                Err(e) => println!("Client connection error: {}", e),
            }
        });
    }
}

pub async fn handle_connection(socket: TcpStream) -> anyhow::Result<()> {
    // Get a stream buffer for the socket
    let stream = tokio::io::BufStream::new(socket);
    let (mut reader, mut writer) = tokio::io::split(stream);

    writer.write_all(&[0xFF, 0xFB, 0x03]).await?;
    writer.flush().await?;

    read_stream(&mut reader, &mut writer).await
}

#[derive(Debug, PartialEq)]
enum TelnetState {
    Normal,
    TelnetCommand,
    WillCommand,
    WontCommand,
    DoCommand,
    DontCommand,
}
async fn read_stream(reader: &mut Reader, _writer: &mut Writer) -> anyhow::Result<()> {
    let mut state = TelnetState::Normal;

    loop {
        if let Ok(byte) = reader.read_u8().await {
            println!("Read byte: {} - State: #{:?}", byte, state);

            state = process_byte(state, byte)?;
        } else {
            return Ok(());
        }
    }
}

fn process_byte(state: TelnetState, byte: u8) -> anyhow::Result<TelnetState> {
    match state {
        TelnetState::Normal => {
            if byte == 0xFF {
                Ok(TelnetState::TelnetCommand)
            } else {
                Ok(TelnetState::Normal)
            }
        }
        TelnetState::TelnetCommand => match byte {
            0xFF => Ok(TelnetState::Normal),
            0xFB => Ok(TelnetState::WillCommand),
            0xFC => Ok(TelnetState::WontCommand),
            0xFD => Ok(TelnetState::DoCommand),
            0xFE => Ok(TelnetState::DontCommand),
            _ => {
                println!("Unknown telnet command: {}", byte);
                Ok(TelnetState::Normal)
            }
        },
        TelnetState::WillCommand => {
            println!("Client will: {}", byte);
            Ok(TelnetState::Normal)
        }
        TelnetState::WontCommand => {
            println!("Client wont: {}", byte);
            Ok(TelnetState::Normal)
        }
        TelnetState::DoCommand => {
            println!("Client do: {}", byte);
            Ok(TelnetState::Normal)
        }
        TelnetState::DontCommand => {
            println!("Client dont: {}", byte);
            Ok(TelnetState::Normal)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_process_byte_normal() -> anyhow::Result<()> {
        let state = TelnetState::Normal;
        let byte = b'a';

        let new_state = process_byte(state, byte)?;

        assert_eq!(new_state, TelnetState::Normal);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_byte_telnet_command() -> anyhow::Result<()> {
        let state = TelnetState::TelnetCommand;
        let byte = 0xFB;

        let new_state = process_byte(state, byte)?;

        assert_eq!(new_state, TelnetState::WillCommand);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_byte_will_command() -> anyhow::Result<()> {
        let state = TelnetState::WillCommand;
        let byte = 0x01;

        let new_state = process_byte(state, byte)?;

        assert_eq!(new_state, TelnetState::Normal);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_byte_unknown_command() -> anyhow::Result<()> {
        let state = TelnetState::TelnetCommand;
        let byte = 0x02;

        let new_state = process_byte(state, byte)?;

        assert_eq!(new_state, TelnetState::Normal);

        Ok(())
    }
}
