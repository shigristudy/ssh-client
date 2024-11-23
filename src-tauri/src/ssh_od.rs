use ssh2::{Session};
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::{Read, Write};
use tokio::time::{self, Duration, Instant};
use log::error;  // Remove debug import

const KEEP_ALIVE_INTERVAL: u64 = 5; // Keep-alive every 5 seconds
const MAX_RETRIES: u32 = 3;

#[allow(dead_code)]
pub struct SessionData {
    session: Session,
    channel: ssh2::Channel,
    last_activity: Instant,
    keep_alive_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConfig {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
}

lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<String, SessionData>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[tauri::command]
pub async fn create_ssh_connection(config: SshConfig) -> Result<String, String> {
    println!("Attempting to connect to {}:{}", config.host, config.port);
    
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .map_err(|e| {
            error!("TCP connection failed: {}", e);
            format!("Connection failed: {}", e)
        })?;

    // Configure TCP
    tcp.set_nodelay(true)
        .map_err(|e| format!("TCP config error: {}", e))?;

    let mut session = Session::new()
        .map_err(|e| format!("Session creation failed: {}", e))?;
    
    session.set_tcp_stream(tcp);
    session.set_timeout(30000); // 30 seconds timeout
    session.set_blocking(true); // Set blocking mode for initial setup
    
    // Initialize session
    if let Err(e) = session.handshake() {
        error!("SSH handshake failed: {}", e);
        return Err(format!("Handshake failed: {}", e));
    }

    println!("Authentication starting...");
    if let Some(private_key) = config.private_key {
        session.userauth_pubkey_memory(
            &config.username,
            None,
            &private_key,
            None
        ).map_err(|e| e.to_string())?;
    } else if let Some(password) = config.password {
        session.userauth_password(&config.username, &password)
            .map_err(|e| e.to_string())?;
    }

    println!("Creating channel...");
    let mut channel = session.channel_session()
        .map_err(|e| {
            println!("Channel creation failed: {}", e);
            e.to_string()
        })?;
    
    println!("Setting up PTY...");
    // Request PTY without modes
    channel.request_pty("xterm-256color", None, None)
        .map_err(|e| {
            println!("PTY request failed: {}", e);
            e.to_string()
        })?;

    println!("Starting shell...");
    channel.shell()
        .map_err(|e| {
            println!("Shell start failed: {}", e);
            e.to_string()
        })?;

    // Wait for shell to be ready
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Set non-blocking after initialization
    session.set_blocking(false);

    let session_id = uuid::Uuid::new_v4().to_string();
    let session_id_clone = session_id.clone();

    SESSIONS.lock().await.insert(session_id.clone(), SessionData { 
        session,
        channel,
        last_activity: Instant::now(),
        keep_alive_count: 0,
    });

    // More aggressive keep-alive check
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL));
        loop {
            interval.tick().await;
            let mut sessions = SESSIONS.lock().await;
            if let Some(session_data) = sessions.get_mut(&session_id_clone) {
                match session_data.session.keepalive_send() {
                    Ok(_) => {
                        session_data.last_activity = Instant::now();
                        session_data.keep_alive_count = 0;
                    },
                    Err(e) => {
                        println!("Keep-alive error for session {}: {}", session_id_clone, e);
                        session_data.keep_alive_count += 1;
                        if session_data.keep_alive_count >= MAX_RETRIES {
                            println!("Session {} removed due to failed keep-alive", session_id_clone);
                            sessions.remove(&session_id_clone);
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
    });

    println!("Connection successful, session ID: {}", session_id);
    Ok(session_id)
}

#[tauri::command]
pub async fn send_ssh_data(session_id: String, data: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().await;
    let session_data = sessions.get_mut(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    // Update activity timestamp on write
    session_data.last_activity = Instant::now();
    
    // Reset keep-alive counter on successful activity
    session_data.keep_alive_count = 0;

    let bytes = data.as_bytes();
    let mut written = 0;
    while written < bytes.len() {
        match session_data.channel.write(&bytes[written..]) {
            Ok(n) => {
                written += n;
                session_data.channel.flush().map_err(|e| e.to_string())?;
            },
            Err(e) => {
                sessions.remove(&session_id);
                return Err(format!("Write error: {}", e));
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn read_ssh_data(session_id: String) -> Result<String, String> {
    let mut sessions = SESSIONS.lock().await;
    let session_data = sessions.get_mut(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    if !session_data.session.authenticated() {
        sessions.remove(&session_id);
        return Err("Connection lost".to_string());
    }

    session_data.last_activity = Instant::now();
    
    let mut buf = vec![0; 8192];
    match session_data.channel.read(&mut buf) {
        Ok(n) if n > 0 => {
            Ok(String::from_utf8_lossy(&buf[..n]).to_string())
        },
        Ok(_) => Ok(String::new()),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            Ok(String::new())
        },
        Err(e) => {
            error!("Read error for session {}: {}", session_id, e);
            sessions.remove(&session_id);
            Err("Connection lost".to_string())
        }
    }
}

#[tauri::command]
pub async fn resize_pty(session_id: String, rows: u32, cols: u32) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().await;
    let session_data = sessions.get_mut(&session_id)
        .ok_or("Session not found")?;
    
    session_data.channel.request_pty_size(cols, rows, None, None)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn close_ssh_connection(session_id: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().await;
    
    if let Some(mut session_data) = sessions.remove(&session_id) {
        // Set blocking and attempt graceful shutdown
        let _ = session_data.session.set_blocking(true);
        
        // Send EOF and wait briefly for acknowledgment
        if let Ok(_) = session_data.channel.send_eof() {
            let _ = tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Close channel
        let _ = session_data.channel.close();
        
        // Brief wait for channel closure
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            async {
                while !session_data.channel.eof() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        ).await;
    }
    
    Ok(())
}
