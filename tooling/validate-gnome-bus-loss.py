#!/usr/bin/env python3
"""Disconnect only a probe's connection to the real session bus.

Run in the GNOME session after building `--example validate_gnome`.
The relay uses the current user's credentials, carries no keyring traffic,
and never disconnects or changes the desktop bus itself.
"""
import os
from pathlib import Path
import selectors
import socket
import subprocess
import tempfile
import threading
from urllib.parse import unquote


def main():
    address = os.environ["DBUS_SESSION_BUS_ADDRESS"].split(";", 1)[0]
    if not address.startswith("unix:path="):
        raise SystemExit("This probe requires a session bus with unix:path")
    upstream_path = unquote(address.removeprefix("unix:path=").split(",", 1)[0])
    binary = Path(__file__).resolve().parents[1] / "src-tauri/target/debug/examples/validate_gnome"
    with tempfile.TemporaryDirectory(prefix="vocalnotes-bus-validation-") as directory:
        relay_path = str(Path(directory) / "relay")
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(relay_path)
        listener.listen(1)
        listener.settimeout(12)
        connections = []
        closed = threading.Event()
        ready = threading.Event()
        failures = []

        def relay():
            try:
                client, _ = listener.accept()
                upstream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                upstream.connect(upstream_path)
                connections.extend([client, upstream])
                ready.set()
                with selectors.DefaultSelector() as selector:
                    selector.register(client, selectors.EVENT_READ, upstream)
                    selector.register(upstream, selectors.EVENT_READ, client)
                    while not closed.is_set():
                        for key, _ in selector.select(0.1):
                            try:
                                data = key.fileobj.recv(65536)
                                if not data:
                                    return
                                key.data.sendall(data)
                            except OSError:
                                if not closed.is_set():
                                    raise
            except Exception as error:
                failures.append(type(error).__name__)

        worker = threading.Thread(target=relay, daemon=True)
        worker.start()
        env = {**os.environ, "DBUS_SESSION_BUS_ADDRESS": "unix:path=" + relay_path}
        process = subprocess.Popen([str(binary), "lifecycle-loss"], env=env,
                                   stdout=subprocess.PIPE, text=True)
        try:
            # The Rust probe has a bounded initialization and loss timeout.
            first = process.stdout.readline().strip()
            if first != "READY lifecycle monitor connected" or not ready.is_set():
                raise RuntimeError("real monitor did not initialize through relay")
            print(first, flush=True)
            closed.set()
            for connection in connections:
                connection.shutdown(socket.SHUT_RDWR)
                connection.close()
            output, _ = process.communicate(timeout=20)
            print(output, end="", flush=True)
            if process.returncode or failures:
                raise RuntimeError("probe failed")
        finally:
            closed.set()
            listener.close()
            for connection in connections:
                connection.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
            worker.join(timeout=2)


if __name__ == "__main__":
    main()
