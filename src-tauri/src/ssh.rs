use ssh2::{Session};
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::{Read, Write};
use tokio::time::{self, Duration};

#[allow(dead_code)]
pub struct SessionData {
    session: Session,
    channel: ssh2::Channel,
    last_activity: std::time::Instant,
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
            println!("TCP connection failed: {}", e);
            e.to_string()
        })?;

    tcp.set_nodelay(true)
        .map_err(|e| e.to_string())?;

    let mut session = Session::new()
        .map_err(|e| e.to_string())?;
    
    session.set_tcp_stream(tcp);
    session.set_timeout(30000); // 30 seconds timeout
    session.set_blocking(true); // Set blocking mode for initial setup
    
    println!("Starting SSH handshake...");
    session.handshake()
        .map_err(|e| {
            println!("Handshake failed: {}", e);
            e.to_string()
        })?;

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
    let session_id_clone = session_id.clone(); // Clone session_id

    SESSIONS.lock().await.insert(session_id.clone(), SessionData { 
        session,
        channel,
        last_activity: std::time::Instant::now(),
    });

    // Start keep-alive task
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let mut sessions = SESSIONS.lock().await;
            if let Some(session_data) = sessions.get_mut(&session_id_clone) {
                if session_data.session.keepalive_send().is_err() {
                    println!("Keep-alive failed for session ID: {}", session_id_clone);
                    sessions.remove(&session_id_clone);
                    break;
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

    if !check_session_health(&session_data.session) {
        sessions.remove(&session_id);
        return Err("Connection lost".to_string());
    }

    let bytes = data.as_bytes();
    let mut written = 0;
    while written < bytes.len() {
        match session_data.channel.write(&bytes[written..]) {
            Ok(n) => {
                written += n;
                session_data.channel.flush().map_err(|e| {
                    println!("Flush error: {}", e);
                    e.to_string()
                })?;
            },
            Err(e) => {
                println!("Write error: {}", e);
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

    // Check session health
    if !check_session_health(&session_data.session) {
        sessions.remove(&session_id);
        return Err("Connection lost".to_string());
    }

    let mut output = String::new();
    let mut buf = [0; 1024];
    
    match session_data.channel.read(&mut buf) {
        Ok(n) if n > 0 => {
            output.push_str(&String::from_utf8_lossy(&buf[..n]));
            session_data.last_activity = std::time::Instant::now();
            Ok(output)
        },
        Ok(_) => Ok(String::new()),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            Ok(String::new())
        },
        Err(e) => {
            let err_string = e.to_string();
            if err_string.contains("transport read") || 
               e.kind() == std::io::ErrorKind::ConnectionReset {
                sessions.remove(&session_id);
                Err("Connection lost".to_string())
            } else {
                Err(format!("Read error: {}", e))
            }
        }
    }
}

fn check_session_health(session: &Session) -> bool {
    // Check if session is alive by attempting a no-op
    session.authenticated() && session.keepalive_send().is_ok()
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
#[allow(dead_code)]
pub async fn close_ssh_connection(session_id: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().await;
    if let Some(mut session_data) = sessions.remove(&session_id) {
        // Close channel first
        session_data.channel.close()
            .map_err(|e| e.to_string())?;
        
        // Wait for channel to close
        session_data.channel.wait_close()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
