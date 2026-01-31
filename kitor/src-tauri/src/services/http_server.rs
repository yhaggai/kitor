use std::net::TcpListener;
use std::io::{Read, Write};
use std::sync::mpsc;

const CALLBACK_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>GOG Login Success</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            background: rgba(255, 255, 255, 0.1);
            padding: 40px;
            border-radius: 20px;
            backdrop-filter: blur(10px);
        }
        .checkmark {
            width: 80px;
            height: 80px;
            border-radius: 50%;
            display: block;
            stroke-width: 3;
            stroke: #fff;
            stroke-miterlimit: 10;
            margin: 0 auto 20px;
            box-shadow: inset 0px 0px 0px #7ac142;
            animation: fill .4s ease-in-out .4s forwards, scale .3s ease-in-out .9s both;
        }
        .checkmark__circle {
            stroke-dasharray: 166;
            stroke-dashoffset: 166;
            stroke-width: 2;
            stroke-miterlimit: 10;
            stroke: #7ac142;
            fill: none;
            animation: stroke 0.6s cubic-bezier(0.65, 0, 0.45, 1) forwards;
        }
        .checkmark__check {
            transform-origin: 50% 50%;
            stroke-dasharray: 48;
            stroke-dashoffset: 48;
            animation: stroke 0.3s cubic-bezier(0.65, 0, 0.45, 1) 0.8s forwards;
        }
        @keyframes stroke {
            100% { stroke-dashoffset: 0; }
        }
        @keyframes scale {
            0%, 100% { transform: none; }
            50% { transform: scale3d(1.1, 1.1, 1); }
        }
        @keyframes fill {
            100% { box-shadow: inset 0px 0px 0px 30px #7ac142; }
        }
        h1 { margin: 0 0 10px; font-size: 28px; }
        p { margin: 0; opacity: 0.9; font-size: 16px; }
    </style>
</head>
<body>
    <div class="container">
        <svg class="checkmark" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 52 52">
            <circle class="checkmark__circle" cx="26" cy="26" r="25" fill="none"/>
            <path class="checkmark__check" fill="none" d="M14.1 27.2l7.1 7.2 16.7-16.8"/>
        </svg>
        <h1>Login Successful!</h1>
        <p>You can close this window now.</p>
    </div>
</body>
</html>"#;

pub fn start_callback_server() -> Result<(mpsc::Receiver<String>, u16), String> {
    let listener = TcpListener::bind("127.0.0.1:7463")
        .map_err(|e| format!("Failed to start callback server: {}", e))?;
    
    let (tx, rx) = mpsc::channel();
    
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            if let Ok(size) = stream.read(&mut buffer) {
                let request = String::from_utf8_lossy(&buffer[..size]);
                
                // Extract code from request
                if let Some(line) = request.lines().next() {
                    if line.starts_with("GET /gog-callback") {
                        if let Some(query_start) = line.find("?code=") {
                            if let Some(query_end) = line[query_start..].find(" HTTP") {
                                let code = &line[query_start + 6..query_start + query_end];
                                let _ = tx.send(code.to_string());
                            }
                        }
                    }
                }
                
                // Send success response
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                    CALLBACK_HTML.len(),
                    CALLBACK_HTML
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        }
    });
    
    Ok((rx, 7463))
}
