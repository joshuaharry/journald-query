use std::fs;
use std::sync::Arc;
use std::collections::HashMap;

use journald_query::{TailConfig, JournalTail};
use poem::{
    get, handler,
    listener::TcpListener,
    web::{
        sse::{Event, SSE},
        Html, Query,
    },
    Route, Server,
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use tokio::sync::{broadcast, RwLock};

#[derive(Deserialize)]
struct LogsQuery {
    hostname: String,
    service: String,
}

/// Serializable wrapper for journal entries (for SSE/JSON output)
#[derive(Serialize, Clone)]
struct SerializableEntry {
    hostname: Option<String>,
    unit: Option<String>,
    timestamp_utc: u64,
    message: String,
}

impl From<journald_query::Entry> for SerializableEntry {
    fn from(entry: journald_query::Entry) -> Self {
        Self {
            hostname: entry.hostname,
            unit: entry.unit,
            timestamp_utc: entry.timestamp_utc,
            message: entry.message,
        }
    }
}

/// Key for identifying unique journal streams
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct StreamKey {
    hostname: String,
    service: String,
}

/// Shared journal reader that multiplexes to multiple connections
struct JournalMultiplexer {
    streams: Arc<RwLock<HashMap<StreamKey, broadcast::Sender<SerializableEntry>>>>,
    machine_id: String,
}

impl JournalMultiplexer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let machine_id = fs::read_to_string("/etc/machine-id")?
            .trim()
            .to_string();
        
        Ok(Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            machine_id,
        })
    }
    
    /// Get or create a broadcast channel for a specific hostname/service combination
    async fn get_or_create_stream(&self, key: StreamKey) -> broadcast::Receiver<SerializableEntry> {
        let mut streams = self.streams.write().await;
        
        if let Some(sender) = streams.get(&key) {
            // Stream already exists, return a new receiver
            return sender.subscribe();
        }
        
        // Create new stream
        let (tx, rx) = broadcast::channel(1000); // Buffer up to 1000 entries
        streams.insert(key.clone(), tx.clone());
        
        // Spawn a single background task for this hostname/service combination
        let journal_path = format!("/var/log/journal/{}", self.machine_id);
        let streams_ref = Arc::clone(&self.streams);
        
        tokio::task::spawn_blocking(move || {
            let config = TailConfig::new(&key.hostname, &key.service, &journal_path)
                .with_poll_interval_ms(100);
            
            let mut tail = match JournalTail::new(config) {
                Ok(tail) => tail,
                Err(e) => {
                    eprintln!("Failed to create journal tail for {:?}: {}", key, e);
                    return;
                }
            };
            
            // Read journal entries and broadcast to all subscribers
            for entry_result in tail.iter() {
                match entry_result {
                    Ok(entry) => {
                        let serializable = SerializableEntry::from(entry);
                        
                        // Send to all subscribers (non-blocking)
                        if tx.send(serializable).is_err() {
                            // No more subscribers, clean up
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Journal error for {:?}: {}", key, e);
                        break;
                    }
                }
            }
            
            // Clean up when done
            tokio::spawn(async move {
                let mut streams = streams_ref.write().await;
                streams.remove(&key);
                println!("Cleaned up stream for {:?}", key);
            });
        });
        
        rx
    }
}

// Global multiplexer instance
static MULTIPLEXER: tokio::sync::OnceCell<JournalMultiplexer> = tokio::sync::OnceCell::const_new();

async fn get_multiplexer() -> &'static JournalMultiplexer {
    MULTIPLEXER.get_or_init(|| async {
        JournalMultiplexer::new().expect("Failed to create multiplexer")
    }).await
}

#[handler]
fn index() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Production-Ready Live Journal Stream</title>
            <style>
                body { font-family: monospace; margin: 20px; }
                .section { 
                    margin-bottom: 30px; 
                    padding: 15px; 
                    border: 1px solid #ddd; 
                    border-radius: 5px;
                    background: #fafafa;
                }
                .log-entry { 
                    margin: 5px 0; 
                    padding: 5px; 
                    border-left: 3px solid #007acc;
                    background: #f5f5f5;
                }
                .timestamp { color: #666; }
                .hostname { color: #0066cc; font-weight: bold; }
                .service { color: #cc6600; }
                .message { color: #000; }
                .controls { margin-bottom: 20px; }
                .status { margin: 10px 0; padding: 10px; background: #e8f4fd; border-radius: 3px; }
                input, button { margin: 5px; padding: 5px; }
            </style>
        </head>
        <body>
            <h1>Production-Ready Live Journal Stream</h1>
            <div class="status">
                <strong>Production Features:</strong>
                ✅ Shared journal readers (no thread-per-connection)<br>
                ✅ Connection multiplexing with broadcast channels<br>
                ✅ Automatic cleanup when connections close<br>
                ✅ Bounded memory usage with buffered channels<br>
            </div>
            
            <div class="section">
                <h2>Live Stream</h2>
                <div class="controls">
                    <input type="text" id="hostname" placeholder="Hostname" value="sdjournal-rs">
                    <input type="text" id="service" placeholder="Service" value="journald-demo.service">
                    <button onclick="startStream()">Start Stream</button>
                    <button onclick="stopStream()">Stop Stream</button>
                    <button onclick="clearLogs()">Clear</button>
                </div>
                <div id="connection-status"></div>
            </div>
            
            <div id="logs"></div>
            
            <script>
                let eventSource = null;
                let connectionCount = 0;
                
                function updateStatus(message, type = 'info') {
                    const status = document.getElementById('connection-status');
                    status.innerHTML = `<strong>Status:</strong> ${message}`;
                    status.style.background = type === 'error' ? '#f8d7da' : '#d4edda';
                    status.style.color = type === 'error' ? '#721c24' : '#155724';
                }
                
                function startStream() {
                    if (eventSource) {
                        eventSource.close();
                    }
                    
                    const hostname = document.getElementById('hostname').value;
                    const service = document.getElementById('service').value;
                    
                    if (!hostname || !service) {
                        alert('Please enter both hostname and service');
                        return;
                    }
                    
                    connectionCount++;
                    updateStatus(`Connecting... (Connection #${connectionCount})`);
                    
                    const url = `/logs?hostname=${encodeURIComponent(hostname)}&service=${encodeURIComponent(service)}`;
                    eventSource = new EventSource(url);
                    
                    eventSource.onopen = function() {
                        updateStatus(`Connected to ${hostname}/${service} (Connection #${connectionCount})`);
                    };
                    
                    eventSource.onmessage = function(event) {
                        try {
                            const entry = JSON.parse(event.data);
                            displayLogEntry(entry);
                        } catch (e) {
                            displayError(event.data);
                        }
                    };
                    
                    eventSource.onerror = function(event) {
                        updateStatus('Connection error occurred', 'error');
                    };
                }
                
                function stopStream() {
                    if (eventSource) {
                        eventSource.close();
                        eventSource = null;
                        updateStatus('Disconnected');
                    }
                }
                
                function clearLogs() {
                    document.getElementById('logs').innerHTML = '';
                }
                
                function displayLogEntry(entry) {
                    const logs = document.getElementById('logs');
                    const div = document.createElement('div');
                    div.className = 'log-entry';
                    
                    const timestamp = new Date(entry.timestamp_utc / 1000).toLocaleString();
                    div.innerHTML = `
                        <span class="timestamp">${timestamp}</span>
                        <span class="hostname">${entry.hostname || 'unknown'}</span>
                        <span class="service">${entry.unit || 'unknown'}</span>
                        <div class="message">${entry.message}</div>
                    `;
                    
                    logs.appendChild(div);
                    logs.scrollTop = logs.scrollHeight;
                    
                    // Limit log entries to prevent memory issues
                    if (logs.children.length > 1000) {
                        logs.removeChild(logs.firstChild);
                    }
                }
                
                function displayError(message) {
                    const logs = document.getElementById('logs');
                    const div = document.createElement('div');
                    div.style.color = 'red';
                    div.textContent = `Error: ${message}`;
                    logs.appendChild(div);
                    logs.scrollTop = logs.scrollHeight;
                }
            </script>
        </body>
        </html>
        "#,
    )
}

#[handler]
async fn logs(Query(params): Query<LogsQuery>) -> Result<SSE, poem::Error> {
    let multiplexer = get_multiplexer().await;
    
    let key = StreamKey {
        hostname: params.hostname,
        service: params.service,
    };
    
    // Get a receiver for this hostname/service combination
    let mut rx = multiplexer.get_or_create_stream(key).await;
    
    // Create async stream from broadcast receiver
    let stream = async_stream::stream! {
        while let Ok(entry) = rx.recv().await {
            match serde_json::to_string(&entry) {
                Ok(json) => yield Event::message(json),
                Err(e) => yield Event::message(format!("Serialization error: {}", e)),
            }
        }
    };
    
    Ok(SSE::new(stream).keep_alive(Duration::from_secs(30)))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    let app = Route::new()
        .at("/", get(index))
        .at("/logs", get(logs));
        
    println!("Server running on http://localhost:3000");
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
