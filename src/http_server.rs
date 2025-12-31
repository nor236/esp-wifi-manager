use core::str;

use embassy_net::tcp::TcpSocket;
use embassy_net::IpListenEndpoint;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
extern crate alloc;
use alloc::vec::Vec;
use alloc::{rc::Rc, string::String};
use esp_println::{print, println};

use crate::structs::{AutoSetupSettings, WmInnerSignals};

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            path: String::new(),
            body: String::new(),
            headers: Vec::new(),
        }
    }

    pub fn parse(data: &[u8]) -> Option<Self> {
        let data_str = str::from_utf8(data).ok()?;
        let mut request = Self::new();
        log::info!("parse: size:={:?} ,data={:?}", data.len(), data_str);

        if let Some((request_line, rest)) = data_str.split_once("\r\n") {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() >= 2 {
                request.method = match parts[0] {
                    "GET" => HttpMethod::Get,
                    "POST" => HttpMethod::Post,
                    _ => return None,
                };
                // log::info!("parse request line:{:?} {:?}", request.method, parts[1]);
                request.path = String::try_from(parts[1]).ok()?;
            }

            if let Some((header_str, body)) = rest.split_once("\r\n\r\n") {
                let header_iter = header_str.split("\r\n");
                for line in header_iter {
                    if let Some((key, value)) = line.split_once(": ") {
                        // log::info!("parse header: {:?} = {:?}", key, value);
                        request
                            .headers
                            .push((String::try_from(key).ok()?, String::try_from(value).ok()?));
                    }
                }
                // log::info!("parse header str: {:?}", header_str);
                // log::info!("parse body: {:?}", body);
                if body.is_empty() {
                    request.body = String::new();
                } else {
                    request.body = String::try_from(body).ok()?;
                }
            }
        }

        Some(request)
    }
}
#[embassy_executor::task]
pub async fn start_http_server(
    stack: embassy_net::Stack<'static>,
    states: Rc<WmInnerSignals>,
) -> ! {
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    loop {
        if stack.is_link_up() {
            log::info!("AP link up");
            break;
        } 
        Timer::after(Duration::from_millis(500)).await;
        log::info!("AP link not up");
    }
    let port = 80;
    let gw_ip_addr_str = states.gw_ip_addr_str.lock().await;
    log::info!(
        "Connect to the AP and point your browser to http://{gw_ip_addr_str}:{port}/",
        gw_ip_addr_str = gw_ip_addr_str.as_str()
    );
    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    stack
        .config_v4()
        .inspect(|c| log::info!("ipv4 config: {c:?}"));

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    // let socket = mk_static!(TcpSocket<'static>, socket);
    loop {
        log::info!("Wait for connection...");
        let r = socket.accept(IpListenEndpoint { addr: None, port }).await;
        log::info!("Connected...");

        if let Err(e) = r {
            log::info!("connect error: {:?}", e);
            Timer::after(Duration::from_millis(100)).await;
            continue;
        }

        let mut buffer = [0u8; 1536];
        let mut pos = 0;

        loop {
            match socket.read(&mut buffer[pos..]).await {
                Ok(0) => {
                    log::info!("read EOF");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };
                    pos += len;
                    if to_print.contains("\r\n\r\n") {
                        print!("reciv: {}", to_print);
                        println!();
                        break;
                    }
                }
                Err(e) => {
                    log::error!("read error: {:?}", e);
                    break;
                }
            };
        }
        let size = pos;

        if let Some(request) = HttpRequest::parse(&buffer[..size]) {
            log::info!("{:?} {:?}", request.method, request.path);
            if request.method == HttpMethod::Get
                && (request.path.as_str() == "/" || request.path.as_str() == "")
            {
                write_html_200(&mut socket, IDX_HTML_DATA).await;
            } else if request.method == HttpMethod::Get && request.path.as_str() == "/list" {
                let resp_res = states.wifi_scan_res.try_lock();
                let resp = match resp_res {
                    Ok(ref resp) => alloc::string::ToString::to_string(resp),
                    Err(_) => String::new(),
                };
                write_text_200(&mut socket, resp.as_bytes()).await;
                log::info!("resp: {:?}", resp);
            } else if request.method == HttpMethod::Post && request.path.as_str() == "/setup" {
                let (ssid, pwd) = parse_form_data(request.body.as_str());
                log::info!("{}/{}", ssid, pwd);
                states
                    .wifi_conn_info_sig
                    .signal(AutoSetupSettings { ssid, psk: pwd });
                write_html_200(&mut socket, SUCCESS_HTML.as_bytes()).await;
            } else if request.method == HttpMethod::Get && request.path.as_str() == "/favicon.ico" {
            } else if request.method == HttpMethod::Get && request.path.as_str() == "/done.html" {
                write_html_200(&mut socket, SUCCESS_HTML.as_bytes()).await;
            } else {
                write_302(&mut socket).await;
            }
        } else {
            log::info!("unknown request");
        }

        socket.close();
        Timer::after(Duration::from_millis(200)).await;
        socket.abort();
    }
}

pub fn generate_res_header(status: u16, content_len: usize, content_type: &str) -> String {
    let response = alloc::format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        content_len,
    );

    response
}

pub fn generate_response(status: u16, content: &str, content_type: &str) -> String {
    let response = alloc::format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        content.len(),
        content
    );

    response
}

pub fn parse_form_data(data: &str) -> (String, String) {
    let mut ssid = String::new();
    let mut pwd = String::new();
    for pair in data.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "ssid" {
                ssid.push_str(value);
            }
            if key == "psk" {
                pwd.push_str(value);
            }
        }
    }
    (ssid, pwd)
}

pub async fn write_302<'d>(socket: &mut embassy_net::tcp::TcpSocket<'d>) {
    let full_response = alloc::format!(
        "HTTP/1.1 302 Found\r\nLocation: /\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let redirect_response = generate_response(302, full_response.as_str(), "text/html");

    let r = socket.write_all(redirect_response.as_bytes()).await;
    if let Err(e) = r {
        log::error!("write chunk error: {:?}", e);
    }
}
pub const IDX_HTML_DATA: &[u8] = include_bytes!("panel.html");

pub async fn write_html_200<'d>(socket: &mut embassy_net::tcp::TcpSocket<'d>, content: &[u8]) {
    write_response(socket, 200, "text/html", content).await;
}

pub async fn write_text_200<'d>(socket: &mut embassy_net::tcp::TcpSocket<'d>, content: &[u8]) {
    write_response(socket, 200, "text/plain", content).await;
}
pub async fn write_response<'d>(
    socket: &mut embassy_net::tcp::TcpSocket<'d>,
    status: u16,
    content_type: &str,
    content: &[u8],
) {
    let res_header = generate_res_header(status, content.len(), content_type);

    //write_all
    let r = socket.write_all(res_header.as_bytes()).await;
    if let Err(e) = r {
        log::error!("write header error: {:?}", e);
    }

    let r = socket.write_all(content).await;
    if let Err(e) = r {
        log::error!("write chunk error: {:?}", e);
    }

    let r = socket.flush().await;
    if let Err(e) = r {
        log::error!("flush  error: {:?}", e);
    }
}

// 配置成功页面
const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <title>配置成功</title>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; max-width: 400px; margin: 0 auto; padding: 20px; text-align: center; }
        h1 { color: #4CAF50; }
    </style>
</head>
<body>
    <h1>配置成功！</h1>
    <p>设备将重新启动并连接到配置的WiFi网络。</p>
</body>
</html>"#;
