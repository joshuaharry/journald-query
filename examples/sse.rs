use std::time::Instant;
use std::fs;

use futures_util::StreamExt;
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

#[derive(Deserialize)]
struct LogsQuery {
    hostname: String,
    service: String,
}

/// Serializable wrapper for journal entries (for SSE/JSON output)
#[derive(Serialize)]
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

#[handler]
fn index() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Live Journal Stream</title>
            <style>
                body { font-family: monospace; margin: 20px; }
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
                input, button { margin: 5px; padding: 5px; }
            </style>
        </head>
        <body>
            <h1>Live Journal Stream</h1>
            <div class="controls">
                <input type="text" id="hostname" placeholder="Hostname (e.g., demo-web-server)" value="sdjournal-rs">
                <input type="text" id="service" placeholder="Service (e.g., journald-demo.service)" value="journald-demo.service">
                <button onclick="startStream()">Start Stream</button>
                <button onclick="stopStream()">Stop Stream</button>
                <button onclick="clearLogs()">Clear</button>
            </div>
            <div id="logs"></div>
            
            <script>
                let eventSource = null;
                
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
                    
                    const url = `/logs?hostname=${encodeURIComponent(hostname)}&service=${encodeURIComponent(service)}`;
                    eventSource = new EventSource(url);
                    
                    eventSource.onopen = function() {
                        console.log('Stream opened');
                    };
                    
                    eventSource.onmessage = function(event) {
                        try {
                            const entry = JSON.parse(event.data);
                            displayLogEntry(entry);
                        } catch (e) {
                            // Handle non-JSON messages (like errors)
                            displayError(event.data);
                        }
                    };
                    
                    eventSource.onerror = function(event) {
                        console.error('Stream error:', event);
                        displayError('Connection error occurred');
                    };
                }
                
                function stopStream() {
                    if (eventSource) {
                        eventSource.close();
                        eventSource = null;
                    }
                    console.log('Stream closed');
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
    // Get the machine ID to construct the correct journal path
    let machine_id = fs::read_to_string("/etc/machine-id")
        .map_err(|e| poem::Error::from_string(format!("Failed to read machine ID: {}", e), poem::http::StatusCode::INTERNAL_SERVER_ERROR))?
        .trim()
        .to_string();
    
    let journal_path = format!("/var/log/journal/{}", machine_id);
    
    // Create tail configuration - use system journal for live demo
    let config = TailConfig::new(&params.hostname, &params.service, &journal_path);
    
    // Convert the blocking iterator to an async stream
    let stream = async_stream::stream! {
        // Move the config to a thread pool for blocking operations
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn blocking task to handle journal iteration
        let _handle = tokio::task::spawn_blocking(move || {
            // Create the journal tail inside the blocking task
            let mut tail = match JournalTail::new(config) {
                Ok(tail) => {
                    tail
                },
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };
            
            // Iterate over journal entries
            for entry_result in tail.iter() {
                match entry_result {
                    Ok(entry) => {
                        if let Err(_) = tx.send(Ok(entry)) {
                            // Channel closed, stop iteration
                            break;
                        }
                    }
                    Err(e) => {
                        if let Err(_) = tx.send(Err(e)) {
                            // Channel closed, stop iteration
                            break;
                        }
                    }
                }
            }
        });
        
        // Yield entries from the channel
        while let Some(result) = rx.recv().await {
            match result {
                Ok(entry) => {
                    let serialized_entry = SerializableEntry::from(entry);
                    // Serialize entry to JSON
                    match serde_json::to_string(&serialized_entry) {
                        Ok(json) => yield Event::message(json),
                        Err(e) => yield Event::message(format!("Serialization error: {}", e)),
                    }
                }
                Err(e) => {
                    yield Event::message(format!("Journal error: {}", e));
                }
            }
        }
    };
    
    Ok(SSE::new(stream).keep_alive(Duration::from_secs(30)))
}

// Keep the old demo event handler for testing
#[handler]
fn demo_event() -> SSE {
    let now = Instant::now();
    SSE::new(
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| Event::message(now.elapsed().as_secs().to_string())),
    )
    .keep_alive(Duration::from_secs(5))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    let app = Route::new()
        .at("/", get(index))
        .at("/logs", get(logs))
        .at("/demo", get(demo_event));
        
    println!("Server starting at http://0.0.0.0:3000");
    println!("Open your browser and go to http://localhost:3000");
    println!("Try hostname: demo-web-server, service: journald-demo.service");
    println!("(Make sure to run ./demo_service/install.sh first!)");
    
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}