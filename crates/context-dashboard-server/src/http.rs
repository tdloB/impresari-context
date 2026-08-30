// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    io::{Read as _, Write as _},
    net::TcpStream,
};

use crate::{DashboardServerError, DashboardServerErrorCode};

pub(crate) const MAX_HEADER_BYTES: usize = 16_384;
pub(crate) const MAX_BODY_BYTES: usize = 262_144;

pub(crate) struct Request {
    pub method: String,
    pub target: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

pub(crate) fn read_request(stream: &mut TcpStream) -> Result<Request, DashboardServerError> {
    let mut received = Vec::with_capacity(2_048);
    let header_end = loop {
        if received.len() >= MAX_HEADER_BYTES {
            return Err(protocol_error());
        }
        let mut chunk = [0_u8; 2_048];
        let count = stream.read(&mut chunk).map_err(|_| protocol_error())?;
        if count == 0 {
            return Err(protocol_error());
        }
        received.extend_from_slice(&chunk[..count]);
        if let Some(index) = received.windows(4).position(|part| part == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let header_bytes = &received[..header_end];
    let header_text = std::str::from_utf8(header_bytes).map_err(|_| protocol_error())?;
    if !header_text.is_ascii() || header_text.contains('\0') {
        return Err(protocol_error());
    }
    let mut lines = header_text[..header_text.len() - 4].split("\r\n");
    let request_line = lines.next().ok_or_else(protocol_error)?;
    let parts = request_line.split(' ').collect::<Vec<_>>();
    if parts.len() != 3
        || parts[0].is_empty()
        || !parts[0].bytes().all(|byte| byte.is_ascii_uppercase())
        || !parts[1].starts_with('/')
        || parts[1].contains(['?', '#'])
        || parts[2] != "HTTP/1.1"
    {
        return Err(protocol_error());
    }
    let mut headers = BTreeMap::new();
    for line in lines {
        if headers.len() >= 64 {
            return Err(protocol_error());
        }
        let (name, value) = line.split_once(':').ok_or_else(protocol_error)?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(protocol_error());
        }
        let name = name.to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty()
            || value
                .bytes()
                .any(|byte| byte < b' ' || byte == 0x7f || !byte.is_ascii())
            || headers.insert(name, value.to_owned()).is_some()
        {
            return Err(protocol_error());
        }
    }
    if headers.contains_key("transfer-encoding")
        || headers
            .keys()
            .any(|name| name == "forwarded" || name.starts_with("x-forwarded-"))
    {
        return Err(protocol_error());
    }
    let content_length = headers
        .get("content-length")
        .map(|value| value.parse::<usize>().map_err(|_| protocol_error()))
        .transpose()?
        .unwrap_or(0);
    if content_length > MAX_BODY_BYTES {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::ResourceLimit,
        ));
    }
    if parts[0] == "POST" && !headers.contains_key("content-length") {
        return Err(protocol_error());
    }
    let already_read = received.len() - header_end;
    if already_read > content_length {
        return Err(protocol_error());
    }
    let mut body = received[header_end..].to_vec();
    body.resize(content_length, 0);
    stream
        .read_exact(&mut body[already_read..])
        .map_err(|_| protocol_error())?;
    Ok(Request {
        method: parts[0].to_owned(),
        target: parts[1].to_owned(),
        headers,
        body,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Response<'a> {
    pub status: u16,
    pub content_type: &'static str,
    pub body: &'a [u8],
}

pub(crate) fn write_response(
    stream: &mut TcpStream,
    response: Response<'_>,
) -> Result<(), DashboardServerError> {
    let reason = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        421 => "Misdirected Request",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let mut header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}Connection: close\r\n",
        response.status,
        reason,
        response.content_type,
        response.body.len(),
        security_headers()
    );
    header.push_str("\r\n");
    stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(response.body))
        .and_then(|()| stream.flush())
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))
}

pub(crate) fn write_sse_header(stream: &mut TcpStream) -> Result<(), DashboardServerError> {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n{}Connection: close\r\n\r\n",
        security_headers()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.flush())
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))
}

pub(crate) fn security_headers() -> &'static str {
    "Cache-Control: no-store\r\n\
Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'none'; font-src 'none'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'\r\n\
Cross-Origin-Embedder-Policy: require-corp\r\n\
Cross-Origin-Opener-Policy: same-origin\r\n\
Cross-Origin-Resource-Policy: same-origin\r\n\
Permissions-Policy: camera=(), geolocation=(), microphone=(), payment=(), usb=()\r\n\
Referrer-Policy: no-referrer\r\n\
X-Content-Type-Options: nosniff\r\n\
X-Frame-Options: DENY\r\n"
}

fn protocol_error() -> DashboardServerError {
    DashboardServerError::new(DashboardServerErrorCode::Protocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{Ipv4Addr, SocketAddrV4, TcpListener},
        thread,
        time::Duration,
    };

    fn parse(bytes: &[u8]) -> Result<Request, DashboardServerError> {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let address = listener.local_addr().expect("address");
        let input = bytes.to_vec();
        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(address).expect("connect");
            stream.write_all(&input).expect("write");
        });
        let (mut server, _) = listener.accept().expect("accept");
        server
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("timeout");
        let result = read_request(&mut server);
        client.join().expect("client");
        result
    }

    #[test]
    fn strict_parser_accepts_one_bounded_request() {
        let request = parse(b"POST /api/policy/apply HTTP/1.1\r\nHost: 127.0.0.1:1\r\nContent-Length: 2\r\nContent-Type: application/json\r\n\r\n{}").expect("request");
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/api/policy/apply");
        assert_eq!(request.body, b"{}");
    }

    #[test]
    fn strict_parser_rejects_ambiguous_or_forwarded_requests() {
        for bytes in [
            b"GET /?token=x HTTP/1.1\r\nHost: x\r\n\r\n".as_slice(),
            b"GET / HTTP/1.0\r\nHost: x\r\n\r\n".as_slice(),
            b"GET / HTTP/1.1\r\nHost: x\r\nHost: y\r\n\r\n".as_slice(),
            b"GET / HTTP/1.1\r\nHost: x\r\nForwarded: host=y\r\n\r\n".as_slice(),
            b"POST /api/shutdown HTTP/1.1\r\nHost: x\r\n\r\n".as_slice(),
        ] {
            assert!(parse(bytes).is_err());
        }
    }
}
