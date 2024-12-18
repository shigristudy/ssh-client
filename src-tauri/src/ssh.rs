use ssh2::{Session, Channel};
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::{Read, Write};
use tokio::time::{self, Duration, Instant};
use log::{error, warn, info};
use std::sync::atomic::{AtomicU32, Ordering};

const KEEP_ALIVE_INTERVAL: u64 = 5;
const MAX_RETRIES: u32 = 3;
const BUFFER_SIZE: usize = 8192;
const CONNECTION_TIMEOUT: u32 = 30;

pub struct SessionData {
    session: Session,
    channel: Channel,
    last_activity: Instant,
    keep_alive_count: AtomicU32,
    buffer: Vec<u8>,
}

// Implement custom Debug for SessionData
impl std::fmt::Debug for SessionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionData")
            .field("last_activity", &self.last_activity)
            .field("keep_alive_count", &self.keep_alive_count)
            .field("buffer_size", &self.buffer.len())
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
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

impl SessionData {
    fn new(session: Session, channel: Channel) -> Self {
        SessionData {
            session,
            channel,
            last_activity: Instant::now(),
            keep_alive_count: AtomicU32::new(0),
            buffer: Vec::with_capacity(BUFFER_SIZE),
        }
    }

    fn update_activity(&self) {
        self.keep_alive_count.store(0, Ordering::SeqCst);
    }
}

#[tauri::command]
pub async fn create_ssh_connection(config: SshConfig) -> Result<String, String> {
    info!("Attempting to connect to {}:{}", config.host, config.port);
    
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .map_err(|e| {
            error!("TCP connection failed: {}", e);
            format!("Connection failed: {}", e)
        })?;

    tcp.set_nodelay(true)
        .map_err(|e| format!("TCP config error: {}", e))?;

    let mut session = Session::new()
        .map_err(|e| format!("Session creation failed: {}", e))?;
    
    session.set_tcp_stream(tcp);
    session.set_timeout(CONNECTION_TIMEOUT * 1000);
    session.set_blocking(true);
    
    if let Err(e) = session.handshake() {
        error!("SSH handshake failed: {}", e);
        return Err(format!("Handshake failed: {}", e));
    }

    info!("Authentication starting...");
    match (&config.private_key, &config.password) {
        (Some(key), _) => {
            session.userauth_pubkey_memory(
                &config.username,
                None,
                key,
                None
            ).map_err(|e| e.to_string())?;
        },
        (_, Some(pass)) => {
            session.userauth_password(&config.username, pass)
                .map_err(|e| e.to_string())?;
        },
        _ => return Err("No authentication method provided".to_string())
    }

    let mut channel = session.channel_session()
        .map_err(|e| {
            error!("Channel creation failed: {}", e);
            e.to_string()
        })?;
    
    channel.request_pty("xterm-256color", None, None)
        .map_err(|e| {
            error!("PTY request failed: {}", e);
            e.to_string()
        })?;

    channel.shell()
        .map_err(|e| {
            error!("Shell start failed: {}", e);
            e.to_string()
        })?;

    std::thread::sleep(std::time::Duration::from_millis(100));
    session.set_blocking(false);

    let session_id = uuid::Uuid::new_v4().to_string();
    let session_id_clone = session_id.clone();

    let session_data = SessionData::new(session, channel);
    SESSIONS.lock().await.insert(session_id.clone(), session_data);

    // Keep-alive handler
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL));
        loop {
            interval.tick().await;
            let mut sessions = SESSIONS.lock().await;
            if let Some(session_data) = sessions.get_mut(&session_id_clone) {
                match session_data.session.keepalive_send() {
                    Ok(_) => {
                        session_data.update_activity();
                    },
                    Err(e) => {
                        warn!("Keep-alive error for session {}: {}", session_id_clone, e);
                        let count = session_data.keep_alive_count.fetch_add(1, Ordering::SeqCst);
                        if count >= MAX_RETRIES {
                            error!("Session {} removed due to failed keep-alive", session_id_clone);
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

    info!("Connection successful, session ID: {}", session_id);
    Ok(session_id)
}

#[tauri::command]
pub async fn send_ssh_data(session_id: String, data: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().await;
    let session_data = sessions.get_mut(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    session_data.update_activity();
    
    let bytes = data.as_bytes();
    let mut written = 0;
    while written < bytes.len() {
        match session_data.channel.write(&bytes[written..]) {
            Ok(n) => {
                written += n;
                session_data.channel.flush().map_err(|e| e.to_string())?;
            },
            Err(e) => {
                error!("Write error for session {}: {}", session_id, e);
                sessions.remove(&session_id);
                return Err("Connection lost".to_string());
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
        error!("Session {} authentication lost", session_id);
        sessions.remove(&session_id);
        return Err("Connection lost".to_string());
    }

    session_data.update_activity();
    
    let mut buffer = vec![0; BUFFER_SIZE];
    match session_data.channel.read(&mut buffer) {
        Ok(n) if n > 0 => {
            // Combine with existing buffer
            session_data.buffer.extend_from_slice(&buffer[..n]);
            
            // Try to decode complete UTF-8 sequences
            match String::from_utf8(session_data.buffer.clone()) {
                Ok(data) => {
                    session_data.buffer.clear();
                    Ok(data)
                },
                Err(e) => {
                    // Keep incomplete UTF-8 sequences in buffer
                    let valid_up_to = e.utf8_error().valid_up_to();
                    if valid_up_to > 0 {
                        let valid_data = String::from_utf8_lossy(&session_data.buffer[..valid_up_to]).to_string();
                        session_data.buffer = session_data.buffer[valid_up_to..].to_vec();
                        Ok(valid_data)
                    } else {
                        Ok(String::new())
                    }
                }
            }
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