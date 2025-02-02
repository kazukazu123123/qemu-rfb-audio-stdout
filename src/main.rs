use clap::Parser;
use std::io::{stderr, Read, Result, Write};
use std::net::TcpStream;
use std::process;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "127.0.0.1")]
    address: String,
    #[clap(short, long, default_value = "5900")]
    port: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let server_address = format!("{}:{}", args.address, args.port);

    eprintln!("EVT.CONNECTING");
    eprintln!("EVT.LOG Connecting to {}", server_address);
    let mut stream = match TcpStream::connect(&server_address) {
        Ok(stream) => {
            eprintln!("EVT.CONNECTED");
            eprintln!("EVT.LOG Connected to {}", server_address);
            stream
        }
        Err(e) => {
            eprintln!("EVT.ERROR_LOG Failed to connect: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = handle_server_communication(&mut stream) {
        eprintln!("EVT.ERROR_LOG Error during communication: {}", e);
        process::exit(1);
    }

    eprintln!("EVT.CONN_CLOSED Connection closed.");
    process::exit(0);
}

fn handle_server_communication(stream: &mut TcpStream) -> Result<()> {
    // Server's protocol version
    let mut version_buffer = [0; 12];
    if let Err(e) = stream.read_exact(&mut version_buffer) {
        eprintln!("EVT.ERROR_LOG Failed to read server version: {}", e);
        process::exit(1);
    }
    let server_version = String::from_utf8_lossy(&version_buffer);
    eprintln!("EVT.LOG Server version: {}", server_version.trim());

    // Send client's version
    let client_version = b"RFB 003.008\n";
    if let Err(e) = stream.write_all(client_version) {
        eprintln!("EVT.ERROR_LOG Failed to send client version: {}", e);
        process::exit(1);
    }
    eprintln!("EVT.LOG Sent client version: RFB 003.008");

    // Security type negotiation
    let mut number_of_security_types = [0; 1];
    if let Err(e) = stream.read_exact(&mut number_of_security_types) {
        eprintln!("EVT.ERROR_LOG Failed to read security types count: {}", e);
        process::exit(1);
    }
    let number_of_security_types = number_of_security_types[0];
    let mut security_types = vec![0; number_of_security_types as usize];
    if let Err(e) = stream.read_exact(&mut security_types) {
        eprintln!("EVT.ERROR_LOG Failed to read security types: {}", e);
        process::exit(1);
    }

    eprintln!(
        "EVT.LOG Number of security types: {}",
        number_of_security_types
    );
    eprintln!("EVT.LOG Security types: {:?}", security_types);

    // Select security type 1 if available
    if security_types.contains(&1) {
        let selected_security_type = 1;
        if let Err(e) = stream.write_all(&[selected_security_type]) {
            eprintln!("EVT.ERROR_LOG Failed to send security type: {}", e);
            process::exit(1);
        }
        eprintln!(
            "EVT.LOG Sent selected security type: {}",
            selected_security_type
        );
    } else {
        eprintln!("EVT.ERROR_LOG Error: Security type 1 is not available.");
        process::exit(1);
    }

    // Security result handling
    let mut security_result = [0; 4];
    if let Err(e) = stream.read_exact(&mut security_result) {
        eprintln!("EVT.ERROR_LOG Failed to read security result: {}", e);
        process::exit(1);
    }

    // Check security result
    if let Err(e) = check_security_result(stream, &security_result) {
        eprintln!("EVT.ERROR_LOG {}", e);
        process::exit(1);
    }

    eprintln!("EVT.LOG Security authentication succeeded");

    // ClientInit: send shared-flag (1 byte)
    let shared_flag: u8 = 1;
    if let Err(e) = stream.write_all(&[shared_flag]) {
        eprintln!("EVT.ERROR_LOG Failed to send shared flag: {}", e);
        process::exit(1);
    }
    eprintln!("EVT.LOG Sent shared-flag: {}", shared_flag);

    // ServerInit: receive framebuffer size, pixel format, and desktop name
    let mut framebuffer_width = [0; 2];
    let mut framebuffer_height = [0; 2];
    if let Err(e) = stream.read_exact(&mut framebuffer_width) {
        eprintln!("EVT.ERROR_LOG Failed to read framebuffer width: {}", e);
        process::exit(1);
    }
    if let Err(e) = stream.read_exact(&mut framebuffer_height) {
        eprintln!("EVT.ERROR_LOG Failed to read framebuffer height: {}", e);
        process::exit(1);
    }
    let framebuffer_width = u16::from_be_bytes(framebuffer_width);
    let framebuffer_height = u16::from_be_bytes(framebuffer_height);
    eprintln!(
        "EVT.LOG Received framebuffer size: {}x{}",
        framebuffer_width, framebuffer_height
    );

    let mut pixel_format = [0; 16];
    if let Err(e) = stream.read_exact(&mut pixel_format) {
        eprintln!("EVT.ERROR_LOG Failed to read pixel format: {}", e);
        process::exit(1);
    }
    eprintln!("EVT.LOG Received pixel format: {:?}", pixel_format);

    let mut name_length_bytes = [0; 4];
    if let Err(e) = stream.read_exact(&mut name_length_bytes) {
        eprintln!("EVT.ERROR_LOG Failed to read desktop name length: {}", e);
        process::exit(1);
    }
    let name_length = u32::from_be_bytes(name_length_bytes);
    let mut name_string = vec![0; name_length as usize];
    if let Err(e) = stream.read_exact(&mut name_string) {
        eprintln!("EVT.ERROR_LOG Failed to read desktop name: {}", e);
        process::exit(1);
    }
    let name_string = String::from_utf8_lossy(&name_string);
    eprintln!("EVT.LOG Received desktop name: {}", name_string);

    // Send SetEncodings message with QEMU Audio encoding
    if let Err(e) = send_set_encodings_qemu_audio(stream) {
        eprintln!("EVT.ERROR_LOG Failed to send encodings: {}", e);
        process::exit(1);
    }

    // Set audio sample format
    if let Err(e) = set_audio_sample_format(stream, 3, 2, 48000) {
        eprintln!("EVT.ERROR_LOG Failed to set audio format: {}", e);
        process::exit(1);
    }

    // Enable audio capture
    if let Err(e) = enable_audio_capture(stream) {
        eprintln!("EVT.ERROR_LOG Failed to enable audio capture: {}", e);
        process::exit(1);
    }

    // Handle server messages
    loop {
        let mut message_type = [0; 1];
        match stream.read_exact(&mut message_type) {
            Ok(_) => match message_type[0] {
                0 => {
                    if let Err(e) = handle_framebuffer_update(stream) {
                        eprintln!("EVT.ERROR_LOG Error handling framebuffer update: {}", e);
                        return Err(e);
                    }
                }
                255 => {
                    if let Err(e) = handle_qemu_audio_message(stream) {
                        eprintln!("EVT.ERROR_LOG Error handling audio message: {}", e);
                        return Err(e);
                    }
                }
                _ => {
                    eprintln!("EVT.ERROR_LOG Unknown message type: {}", message_type[0]);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Unknown message type",
                    ));
                }
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    eprintln!("EVT.LOG Server closed the connection.");
                    return Ok(());
                } else {
                    eprintln!("EVT.ERROR_LOG Failed to read message type: {}", e);
                    return Err(e);
                }
            }
        }
    }
}

fn handle_framebuffer_update(stream: &mut TcpStream) -> Result<()> {
    let mut padding = [0; 1];
    stream.read_exact(&mut padding)?;

    let mut num_rectangles = [0; 2];
    stream.read_exact(&mut num_rectangles)?;
    let num_rectangles = u16::from_be_bytes(num_rectangles);

    for _ in 0..num_rectangles {
        let mut rect_header = [0; 12];
        stream.read_exact(&mut rect_header)?;
    }

    Ok(())
}

fn handle_qemu_audio_message(stream: &mut TcpStream) -> Result<()> {
    let mut submessage_type = [0; 1];
    stream.read_exact(&mut submessage_type)?;

    let mut operation = [0; 2];
    stream.read_exact(&mut operation)?;
    let operation = u16::from_be_bytes(operation);

    match operation {
        0 => writeln!(stderr(), "EVT.AUDIOSTOP")?,
        1 => writeln!(stderr(), "EVT.AUDIOSTART")?,
        2 => {
            let mut data_length_bytes = [0; 4];
            stream.read_exact(&mut data_length_bytes)?;
            let data_length = u32::from_be_bytes(data_length_bytes);

            let mut data = vec![0; data_length as usize];
            stream.read_exact(&mut data)?;

            let stdout = std::io::stdout();
            let mut stdout_handle = stdout.lock();
            stdout_handle.write_all(&data)?;
            stdout_handle.flush()?;
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "EVT.ERROR_LOG Unknown operation in QEMU Audio Server Message",
            ));
        }
    }

    Ok(())
}

// Security result check
fn check_security_result(stream: &mut TcpStream, result: &[u8; 4]) -> Result<()> {
    if result != &[0, 0, 0, 0] {
        // Authentication failed, get failure reason
        let reason = handle_security_failure(stream)?;
        eprintln!(
            "EVT.ERROR_LOG Security authentication failed. Reason: {}",
            reason
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Security authentication failed",
        ));
    }
    Ok(())
}

// Handle security failure and get the reason
fn handle_security_failure(stream: &mut TcpStream) -> Result<String> {
    let mut reason_length = [0; 4];
    stream.read_exact(&mut reason_length)?;
    let reason_length = u32::from_be_bytes(reason_length);

    let mut reason = vec![0; reason_length as usize];
    stream.read_exact(&mut reason)?;

    match String::from_utf8(reason) {
        Ok(reason_str) => Ok(reason_str),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid UTF-8 in failure reason",
        )),
    }
}

// Send SetEncodings message with QEMU Audio encoding
fn send_set_encodings_qemu_audio(stream: &mut TcpStream) -> Result<()> {
    // SetEncodings message structure
    let message_type: u8 = 2; // SetEncodings message type
    let padding: u8 = 0; // Padding byte
    let encodings = vec![-259]; // QEMU Audio encoding type
    let number_of_encodings = encodings.len() as u16;

    // Send message-type, padding, and number-of-encodings
    stream.write_all(&[message_type])?;
    stream.write_all(&[padding])?;
    stream.write_all(&number_of_encodings.to_be_bytes())?;

    // Send each encoding type
    for &encoding in &encodings {
        stream.write_all(&(encoding as i32).to_be_bytes())?;
    }

    eprintln!("EVT.LOG Sent SetEncodings message with QEMU Audio encoding.");

    Ok(())
}

fn enable_audio_capture(stream: &mut TcpStream) -> Result<()> {
    // QEMU Audio Client Message: Enable Audio Capture
    let message_type: u8 = 255; // Message type for QEMU Audio Client Message
    let submessage_type: u8 = 1; // Submessage type for Audio Control
    let operation: u16 = 0; // Operation 0 to enable audio capture

    let mut buffer = Vec::new();

    // Construct message for audio capture enablement
    buffer.push(message_type);
    buffer.push(submessage_type);
    buffer.extend_from_slice(&operation.to_be_bytes());

    // Send the message to enable audio capture
    stream.write_all(&buffer)?;

    eprintln!("EVT.LOG Sent enable audio capture request.");
    Ok(())
}

// Set the audio sample format
fn set_audio_sample_format(
    stream: &mut TcpStream,
    format: u8,
    channels: u8,
    rate: u32,
) -> Result<()> {
    // QEMU Audio Client Message: Set Audio Sample Format
    let message_type: u8 = 255; // Message type for QEMU Audio Client Message
    let submessage_type: u8 = 1; // Submessage type for Audio Control
    let operation: u16 = 2; // Operation 2 for setting sample format

    let mut buffer = Vec::new();

    // Construct the message for setting audio sample format
    buffer.push(message_type);
    buffer.push(submessage_type);
    buffer.extend_from_slice(&operation.to_be_bytes());
    buffer.push(format);
    buffer.push(channels);
    buffer.extend_from_slice(&(rate).to_be_bytes());

    // Send the message to set the audio sample format
    stream.write_all(&buffer)?;

    eprintln!(
        "EVT.LOG Sent QEMU Audio Client Message with operation 2 (Set Audio Sample Format): \
        Sample Format: {}, Channels: {}, Frequency: {}",
        format, channels, rate
    );
    Ok(())
}
