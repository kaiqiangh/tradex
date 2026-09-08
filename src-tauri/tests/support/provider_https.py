"""Disposable loopback CONNECT/TLS fixture. Never loads real credentials or trust stores."""
from pathlib import Path
import socket
import ssl
import subprocess
import sys
import time

mode, directory = sys.argv[1], Path(sys.argv[2])
host = sys.argv[3]
assert host in {"paper-api.alpaca.markets", "demo.trading212.com", "live.trading212.com"}
path = "/v2/account" if host == "paper-api.alpaca.markets" else "/api/v0/equity/account/summary"
key, cert = directory / "key.pem", directory / "cert.pem"
subprocess.run([
    "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
    "-subj", f"/CN={host}", "-addext", f"subjectAltName=DNS:{host}",
    "-addext", "extendedKeyUsage=serverAuth",
    "-keyout", str(key), "-out", str(cert),
], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(cert, key)


def headers(connection):
    data = b""
    while b"\r\n\r\n" not in data:
        part = connection.recv(4096)
        if not part or len(data) + len(part) > 16384:
            raise ValueError("Incomplete or oversized test request")
        data += part
    return data


with socket.socket() as listener:
    listener.bind(("127.0.0.1", 0))
    listener.listen()
    print(listener.getsockname()[1], flush=True)
    count = 0
    while True:
        raw, _ = listener.accept()
        raw.settimeout(20)
        try:
            assert headers(raw).split(b"\r\n", 1)[0] == f"CONNECT {host}:443 HTTP/1.1".encode()
            raw.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            with context.wrap_socket(raw, server_side=True) as connection:
                request = headers(connection)
                assert request.split(b"\r\n", 1)[0] == f"GET {path} HTTP/1.1".encode()
                assert b"apca-api-key-id: synthetic-network-test" in request.lower()
                count += 1
                (directory / "requests").write_text(str(count))
                if mode == "timeout":
                    time.sleep(20)
                elif mode == "large":
                    connection.sendall(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n" + b"x" * (2 * 1024 * 1024 + 1))
                else:
                    status = {"redirect": b"302 Found", "auth": b"401 Unauthorized", "rate": b"429 Too Many Requests", "ok": b"200 OK"}[mode]
                    location = b"Location: https://paper-api.alpaca.markets/v2/account\r\n" if mode == "redirect" else b""
                    connection.sendall(b"HTTP/1.1 " + status + b"\r\n" + location + b"Content-Length: 2\r\nConnection: close\r\n\r\n{}")
        except (ConnectionError, ssl.SSLError, TimeoutError):
            raw.close()
