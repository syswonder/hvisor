#!/usr/bin/env python3
"""Unified terminal helpers for QEMU UNIX socket and physical serial ports."""

from __future__ import annotations

import queue
import re
import select
import socket
import threading
import time
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path

import serial

RUN_RESULT_RE = re.compile(
    r"__CI_RUN_RESULT__ case=(?P<case>[^\s]+) run_id=(?P<run_id>[a-f0-9]+) rc=(?P<rc>\d+)"
)


class TerminalTimeoutError(TimeoutError):
    """Raised when terminal command wait times out."""


class TerminalCommandError(RuntimeError):
    """Raised when a terminal command exits with non-zero status."""


class TerminalBackend(ABC):
    """Backend abstraction for terminal IO."""

    @abstractmethod
    def open(self) -> None:
        pass

    @abstractmethod
    def close(self) -> None:
        pass

    @abstractmethod
    def read(self, max_bytes: int = 4096) -> bytes:
        pass

    @abstractmethod
    def write(self, data: bytes) -> None:
        pass

    @abstractmethod
    def flush_input(self) -> None:
        pass


@dataclass
class QemuSocketBackend(TerminalBackend):
    path: str
    connect_timeout: float = 10.0
    io_timeout: float = 0.2
    _sock: socket.socket | None = None

    def open(self) -> None:
        if self._sock is not None:
            return
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(self.connect_timeout)
        sock.connect(self.path)
        sock.setblocking(False)
        self._sock = sock

    def close(self) -> None:
        if self._sock is None:
            return
        self._sock.close()
        self._sock = None

    def read(self, max_bytes: int = 4096) -> bytes:
        if self._sock is None:
            raise RuntimeError("QEMU socket is not open")
        ready, _, _ = select.select([self._sock], [], [], self.io_timeout)
        if not ready:
            return b""
        try:
            return self._sock.recv(max_bytes)
        except BlockingIOError:
            return b""

    def write(self, data: bytes) -> None:
        if self._sock is None:
            raise RuntimeError("QEMU socket is not open")
        self._sock.sendall(data)

    def flush_input(self) -> None:
        if self._sock is None:
            return
        while True:
            chunk = self.read()
            if not chunk:
                break


@dataclass
class SerialBackend(TerminalBackend):
    port: str
    baudrate: int = 115200
    timeout: float = 0.2
    _serial: serial.Serial | None = None

    def open(self) -> None:
        if self._serial is not None:
            return
        self._serial = serial.Serial(
            port=self.port,
            baudrate=self.baudrate,
            timeout=self.timeout,
            write_timeout=self.timeout,
        )
        # CH340 adapters often need DTR asserted before the target UART TX is enabled.
        self._serial.dtr = True
        self._serial.rts = False

    def close(self) -> None:
        if self._serial is None:
            return
        self._serial.close()
        self._serial = None

    def read(self, max_bytes: int = 4096) -> bytes:
        if self._serial is None:
            raise RuntimeError("Serial device is not open")
        return self._serial.read(max_bytes)

    def write(self, data: bytes) -> None:
        if self._serial is None:
            raise RuntimeError("Serial device is not open")
        self._serial.write(data)
        self._serial.flush()

    def flush_input(self) -> None:
        if self._serial is None:
            return
        self._serial.reset_input_buffer()


class LogCollector:
    """Background reader that writes terminal output to a log file and console."""

    def __init__(
        self,
        backend: TerminalBackend,
        log_path: Path,
        encoding: str = "utf-8",
        console: bool = True,
        poll_interval: float = 0.05,
    ) -> None:
        self.backend = backend
        self.log_path = log_path
        self.encoding = encoding
        self.console = console
        self.poll_interval = poll_interval
        self._lock = threading.Lock()
        self._buffer = ""
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self._console_queue: queue.SimpleQueue[str | None] = queue.SimpleQueue()
        self._console_thread: threading.Thread | None = None

    def start(self) -> None:
        self.log_path.parent.mkdir(parents=True, exist_ok=True)
        if self.console:
            self._console_thread = threading.Thread(
                target=self._console_loop, name="LogCollectorConsole", daemon=True
            )
            self._console_thread.start()
        self._thread = threading.Thread(target=self._run_loop, name="LogCollector", daemon=True)
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=5.0)
            self._thread = None
        # Drain any bytes still buffered in the backend after the reader exits.
        deadline = time.monotonic() + 1.0
        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if not chunk:
                break
            self._append(chunk.decode(self.encoding, errors="replace"), emit_console=False)
        if self.console:
            self._console_queue.put(None)
            if self._console_thread is not None:
                self._console_thread.join(timeout=5.0)
                self._console_thread = None

    def offset(self) -> int:
        with self._lock:
            return len(self._buffer)

    def tail_since(self, offset: int) -> str:
        with self._lock:
            if offset < 0 or offset > len(self._buffer):
                return self._buffer
            return self._buffer[offset:]

    def text(self) -> str:
        with self._lock:
            return self._buffer

    def _append(self, chunk: str, *, emit_console: bool = True) -> None:
        if not chunk:
            return
        with self._lock:
            self._buffer += chunk
        try:
            with self.log_path.open("a", encoding=self.encoding) as fh:
                fh.write(chunk)
        except OSError:
            pass
        if emit_console and self.console:
            self._console_queue.put(chunk)

    def _console_loop(self) -> None:
        while True:
            chunk = self._console_queue.get()
            if chunk is None:
                return
            print(chunk, end="", flush=True)

    def _run_loop(self) -> None:
        while not self._stop.is_set():
            chunk = self.backend.read()
            if chunk:
                self._append(chunk.decode(self.encoding, errors="replace"))
                continue
            time.sleep(self.poll_interval)


class Terminal:
    """High level terminal wrapper with log-file-driven command helpers."""

    def __init__(
        self,
        backend: TerminalBackend,
        log_path: Path,
        encoding: str = "utf-8",
        console: bool = True,
    ) -> None:
        self.backend = backend
        self.encoding = encoding
        self._collector = LogCollector(backend, log_path, encoding=encoding, console=console)
        self._opened = False

    @classmethod
    def from_qemu_socket(
        cls,
        path: str,
        log_path: Path,
        connect_timeout: float = 10.0,
        io_timeout: float = 0.2,
        encoding: str = "utf-8",
        console: bool = True,
    ) -> "Terminal":
        return cls(
            QemuSocketBackend(path=path, connect_timeout=connect_timeout, io_timeout=io_timeout),
            log_path=log_path,
            encoding=encoding,
            console=console,
        )

    @classmethod
    def from_serial(
        cls,
        port: str,
        log_path: Path,
        baudrate: int = 115200,
        timeout: float = 0.2,
        encoding: str = "utf-8",
        console: bool = True,
    ) -> "Terminal":
        return cls(
            SerialBackend(port=port, baudrate=baudrate, timeout=timeout),
            log_path=log_path,
            encoding=encoding,
            console=console,
        )

    def open(self) -> None:
        if self._opened:
            return
        self.backend.open()
        self._collector.start()
        self._opened = True

    def close(self) -> None:
        if not self._opened:
            return
        self._collector.stop()
        self.backend.close()
        self._opened = False

    def __enter__(self) -> "Terminal":
        self.open()
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    def flush_input(self) -> None:
        self._ensure_open()
        self.backend.flush_input()

    def send(self, command: str) -> None:
        self._ensure_open()
        payload = command.rstrip("\n") + "\n"
        self.backend.write(payload.encode(self.encoding, errors="replace"))

    def run(
        self,
        case: str,
        command: str,
        timeout: float = 30.0,
        poll_interval: float = 0.05,
    ) -> tuple[int, str]:
        """Run a shell command and wait for __CI_RUN_RESULT__ in the log."""
        self._ensure_open()
        run_id = uuid.uuid4().hex
        offset = self._collector.offset()
        wrapped = f"{command}; echo __CI_RUN_RESULT__ case={case} run_id={run_id} rc=$?"
        self.send(wrapped)

        deadline = time.monotonic() + timeout
        run_id_needle = f"run_id={run_id}"
        while time.monotonic() < deadline:
            chunk = self._collector.tail_since(offset)
            if run_id_needle in chunk:
                matches = list(RUN_RESULT_RE.finditer(chunk))
                for match in reversed(matches):
                    if match.group("run_id") == run_id:
                        rc = int(match.group("rc"))
                        output = chunk[: match.start()]
                        return rc, output
            time.sleep(poll_interval)

        raise TerminalTimeoutError(
            f"timed out waiting for run result (case={case}, run_id={run_id}): {command}"
        )

    def wait_pattern(
        self,
        pattern: str,
        timeout: float = 120.0,
        poll_interval: float = 0.05,
        from_offset: int | None = None,
    ) -> bool:
        """Wait until regex pattern appears in the collected log."""
        self._ensure_open()
        offset = self._collector.offset() if from_offset is None else from_offset
        compiled = re.compile(pattern)
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            chunk = self._collector.tail_since(offset)
            if compiled.search(chunk):
                return True
            time.sleep(poll_interval)
        return False

    def _ensure_open(self) -> None:
        if not self._opened:
            self.open()
