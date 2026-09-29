//! Servidor HTTP/1.1 mínimo para tests CardDAV (loopback; sin httpmock).
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
struct Route {
    method: String,
    path: String,
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    once: bool,
    /// Si está, el request debe traer esta cabecera (comparación case-insensitive del valor).
    expect_header: Option<(String, String)>,
    mismatch_status: u16,
}

/// Request capturado (cabeceras) para aserciones.
#[derive(Clone, Debug)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
}

/// Mock HTTP en 127.0.0.1.
pub struct MockHttp {
    pub addr: SocketAddr,
    routes: Arc<Mutex<Vec<Route>>>,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
}

impl MockHttp {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let addr = listener.local_addr().expect("local addr");
        let routes = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::new(Mutex::new(Vec::new()));
        let routes_thread = Arc::clone(&routes);
        let captured_thread = Arc::clone(&captured);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let incoming = match read_request(stream) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                captured_thread
                    .lock()
                    .expect("captured")
                    .push(CapturedRequest {
                        method: incoming.method.clone(),
                        path: incoming.path.clone(),
                        headers: incoming.headers.clone(),
                    });
                let mut routes = routes_thread.lock().expect("routes");
                let _ = respond(&incoming, &mut routes);
            }
        });
        Self {
            addr,
            routes,
            captured,
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn mock(
        &self,
        method: &str,
        path: &str,
        status: u16,
        headers: &[(&str, &str)],
        body: impl AsRef<[u8]>,
    ) {
        self.push_route(method, path, status, headers, body, false, None, 412);
    }

    /// Ruta que se consume tras el primer acierto (secuencias de watch).
    pub fn mock_once(
        &self,
        method: &str,
        path: &str,
        status: u16,
        headers: &[(&str, &str)],
        body: impl AsRef<[u8]>,
    ) {
        self.push_route(method, path, status, headers, body, true, None, 412);
    }

    /// PUT/DELETE: si `If-Match` no coincide, responde `mismatch_status` (típicamente 412).
    pub fn mock_if_match(
        &self,
        method: &str,
        path: &str,
        expected_etag: &str,
        status: u16,
        headers: &[(&str, &str)],
        body: impl AsRef<[u8]>,
    ) {
        self.push_route(
            method,
            path,
            status,
            headers,
            body,
            false,
            Some(("If-Match".into(), expected_etag.to_string())),
            412,
        );
    }

    pub fn last_request(&self) -> Option<CapturedRequest> {
        self.captured.lock().expect("captured").last().cloned()
    }

    pub fn header_of_last(&self, name: &str) -> Option<String> {
        let last = self.last_request()?;
        last.headers.iter().find_map(|(k, v)| {
            if k.eq_ignore_ascii_case(name) {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn push_route(
        &self,
        method: &str,
        path: &str,
        status: u16,
        headers: &[(&str, &str)],
        body: impl AsRef<[u8]>,
        once: bool,
        expect_header: Option<(String, String)>,
        mismatch_status: u16,
    ) {
        self.routes.lock().expect("routes").push(Route {
            method: method.to_ascii_uppercase(),
            path: path.to_string(),
            status,
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            body: body.as_ref().to_vec(),
            once,
            expect_header,
            mismatch_status,
        });
    }
}

struct Incoming {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    stream: TcpStream,
}

fn respond(incoming: &Incoming, routes: &mut Vec<Route>) -> std::io::Result<()> {
    let mut stream = incoming.stream.try_clone()?;
    let found = routes
        .iter()
        .position(|r| r.method == incoming.method && r.path == incoming.path);
    let Some(idx) = found else {
        let not_found = Route {
            method: String::new(),
            path: String::new(),
            status: 404,
            headers: Vec::new(),
            body: b"not found".to_vec(),
            once: false,
            expect_header: None,
            mismatch_status: 412,
        };
        return write_response(&mut stream, &not_found);
    };

    let header_ok = match &routes[idx].expect_header {
        None => true,
        Some((name, expected)) => incoming
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case(name) && etag_eq(v, expected)),
    };

    if !header_ok {
        let mismatch = Route {
            method: String::new(),
            path: String::new(),
            status: routes[idx].mismatch_status,
            headers: Vec::new(),
            body: b"precondition failed".to_vec(),
            once: false,
            expect_header: None,
            mismatch_status: 412,
        };
        return write_response(&mut stream, &mismatch);
    }

    let route = if routes[idx].once {
        routes.remove(idx)
    } else {
        routes[idx].clone()
    };
    write_response(&mut stream, &route)
}

fn etag_eq(got: &str, expected: &str) -> bool {
    normalize_etag(got) == normalize_etag(expected)
}

fn normalize_etag(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn read_request(mut stream: TcpStream) -> std::io::Result<Incoming> {
    let (method, path, headers, body_len) = read_head(&mut stream)?;
    if body_len > 0 {
        let mut rest = vec![0u8; body_len];
        stream.read_exact(&mut rest)?;
    }
    Ok(Incoming {
        method,
        path,
        headers,
        stream,
    })
}

type ParsedHead = (String, String, Vec<(String, String)>, usize);

fn read_head(stream: &mut TcpStream) -> std::io::Result<ParsedHead> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 512];
    loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_double_crlf(&buf) {
            let head = std::str::from_utf8(&buf[..pos]).unwrap_or("");
            let leftover = buf.len() - (pos + 4);
            let (method, path, headers, content_len) = parse_head(head);
            let still = content_len.saturating_sub(leftover);
            return Ok((method, path, headers, still));
        }
        if buf.len() > 64 * 1024 {
            break;
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "cabeceras HTTP incompletas",
    ))
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn parse_head(head: &str) -> ParsedHead {
    let mut lines = head.split("\r\n");
    let req = lines.next().unwrap_or("");
    let mut parts = req.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_ascii_uppercase();
    let path_q = parts.next().unwrap_or("/");
    let path = path_q.split('?').next().unwrap_or("/").to_string();
    let mut content_len = 0usize;
    let mut headers = Vec::new();
    for line in lines {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim().to_string();
        let val = v.trim().to_string();
        if key.eq_ignore_ascii_case("content-length") {
            content_len = val.parse().unwrap_or(0);
        }
        headers.push((key, val));
    }
    (method, path, headers, content_len)
}

fn write_response(stream: &mut TcpStream, route: &Route) -> std::io::Result<()> {
    let reason = match route.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        207 => "Multi-Status",
        301 => "Moved Permanently",
        401 => "Unauthorized",
        404 => "Not Found",
        412 => "Precondition Failed",
        429 => "Too Many Requests",
        _ => "OK",
    };
    let mut out = format!(
        "HTTP/1.1 {} {reason}\r\nConnection: close\r\n",
        route.status
    );
    let mut has_len = false;
    for (k, v) in &route.headers {
        if k.eq_ignore_ascii_case("content-length") {
            has_len = true;
        }
        out.push_str(&format!("{k}: {v}\r\n"));
    }
    if !has_len {
        out.push_str(&format!("Content-Length: {}\r\n", route.body.len()));
    }
    out.push_str("\r\n");
    stream.write_all(out.as_bytes())?;
    stream.write_all(&route.body)?;
    stream.flush()
}
