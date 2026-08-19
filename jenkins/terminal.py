#!/usr/bin/env python3
"""Unified terminal helpers for QEMU UNIX socket and physical serial ports."""

from __future__ import annotations

import os
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


MARKER_ID_LEN = 4
HV_M_MARKER_RE = re.compile(rf"__HV_M_(?P<run_id>[a-f0-9]{{{MARKER_ID_LEN}}}):(?P<rc>\d+)")


def new_marker_id() -> str:
    return uuid.uuid4().hex[:MARKER_ID_LEN]


class TerminalTimeoutError(TimeoutError):
    """Raised when terminal command wait times out."""

    def __init__(self, message: str, *, partial_output: str = "") -> None:
        super().__init__(message)
        self.partial_output = partial_output


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
    """Background reader that continuously captures terminal output."""

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


def _default_log_path() -> Path:
    return Path(f"/tmp/hvisor-terminal-{os.getpid()}.log")


class Terminal:
    """High level terminal wrapper with command helpers."""

    def __init__(
        self,
        backend: TerminalBackend,
        encoding: str = "utf-8",
        log_path: Path | None = None,
        console: bool = True,
    ) -> None:
        self.backend = backend
        self.encoding = encoding
        self._log_path = log_path or _default_log_path()
        self._collector = LogCollector(
            backend, self._log_path, encoding=encoding, console=console
        )
        self._opened = False

    @classmethod
    def from_qemu_socket(
        cls,
        path: str,
        connect_timeout: float = 10.0,
        io_timeout: float = 0.2,
        encoding: str = "utf-8",
        log_path: Path | None = None,
        console: bool = True,
    ) -> "Terminal":
        return cls(
            QemuSocketBackend(path=path, connect_timeout=connect_timeout, io_timeout=io_timeout),
            encoding=encoding,
            log_path=log_path,
            console=console,
        )

    @classmethod
    def from_serial(
        cls,
        port: str,
        baudrate: int = 115200,
        timeout: float = 0.2,
        encoding: str = "utf-8",
        log_path: Path | None = None,
        console: bool = True,
    ) -> "Terminal":
        return cls(
            SerialBackend(port=port, baudrate=baudrate, timeout=timeout),
            encoding=encoding,
            log_path=log_path,
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

    def offset(self) -> int:
        self._ensure_open()
        return self._collector.offset()

    def tail_since(self, offset: int) -> str:
        self._ensure_open()
        return self._collector.tail_since(offset)

    def run(
        self,
        case: str,
        command: str,
        timeout: float = 30.0,
        poll_interval: float = 0.05,
        wake_interval: float | None = None,
    ) -> tuple[int, str]:
        """Run a shell command and wait for the compact result marker in the log."""
        self._ensure_open()
        run_id = new_marker_id()
        offset = self._collector.offset()
        wrapped = f"{command}; echo __HV_M_{run_id}:$?"
        self.send(wrapped)

        deadline = time.monotonic() + timeout
        next_wake = time.monotonic() + wake_interval if wake_interval else None
        marker_needle = rf"__HV_M_{run_id}:\d+"
        while time.monotonic() < deadline:
            chunk = self._collector.tail_since(offset)
            if re.search(marker_needle, chunk):
                matches = [m for m in HV_M_MARKER_RE.finditer(chunk) if m.group("run_id") == run_id]
                if matches:
                    match = matches[-1]
                    rc = int(match.group("rc"))
                    output = chunk[: match.start()]
                    return rc, output
            if next_wake is not None and time.monotonic() >= next_wake:
                self.send("")
                next_wake = time.monotonic() + wake_interval
            time.sleep(poll_interval)

        partial = self._collector.tail_since(offset)
        raise TerminalTimeoutError(
            f"timed out waiting for run result (case={case}, run_id={run_id}): {command}",
            partial_output=partial,
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

    def send(self, command: str) -> None:
        self._ensure_open()
        payload = command.rstrip("\n") + "\n"
        self.backend.write(payload.encode(self.encoding, errors="replace"))

    def read_for(
        self,
        duration: float = 2.0,
        poll_interval: float = 0.05,
    ) -> str:
        self._ensure_open()
        offset = self._collector.offset()
        deadline = time.monotonic() + duration
        while time.monotonic() < deadline:
            time.sleep(poll_interval)
        return self._collector.tail_since(offset)

    def send_until_get(
        self,
        command: str,
        timeout: float = 30.0,
        poll_interval: float = 0.05,
        include_marker_line: bool = False,
    ) -> str:
        self._ensure_open()
        run_id = new_marker_id()
        marker = f"__HV_M_{run_id}:0"
        offset = self._collector.offset()
        self.send(f"{command}; echo {marker}")

        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            buf = self._collector.tail_since(offset)
            if marker in buf:
                if include_marker_line:
                    return buf
                return self._trim_after_marker(buf, marker)
            time.sleep(poll_interval)
        raise TerminalTimeoutError(
            f"timed out waiting for terminal marker: {marker}",
            partial_output=self._collector.tail_since(offset),
        )

    def send_until_quiet(
        self,
        command: str,
        quiet_seconds: float = 1.0,
        max_duration: float = 30.0,
        poll_interval: float = 0.05,
    ) -> str:
        self._ensure_open()
        offset = self._collector.offset()
        self.send(command)
        return self._wait_quiet(offset, quiet_seconds, max_duration, poll_interval, context=command)

    def send_and_drain(
        self,
        command: str,
        read_duration: float = 0.5,
        poll_interval: float = 0.05,
    ) -> str:
        """Send command and collect best-effort output for a fixed duration."""
        self._ensure_open()
        offset = self._collector.offset()
        self.send(command)
        deadline = time.monotonic() + read_duration
        while time.monotonic() < deadline:
            time.sleep(poll_interval)
        return self._collector.tail_since(offset)

    def read_until_quiet(
        self,
        quiet_seconds: float = 3.0,
        max_duration: float = 120.0,
        poll_interval: float = 0.05,
    ) -> str:
        """Continuously read until quiet for x seconds or total timeout."""
        self._ensure_open()
        offset = self._collector.offset()
        try:
            return self._wait_quiet(
                offset, quiet_seconds, max_duration, poll_interval, context="read"
            )
        except TerminalTimeoutError:
            return self._collector.tail_since(offset)

    def run_until_quiet_with_status(
        self,
        command: str,
        quiet_seconds: float = 1.0,
        max_duration: float = 30.0,
        poll_interval: float = 0.05,
    ) -> tuple[str, int]:
        run_id = new_marker_id()
        marker = f"__HV_M_{run_id}"
        wrapped = f"{command}; echo {marker}:$?"
        self._ensure_open()
        offset = self._collector.offset()
        self.send(wrapped)

        deadline = time.monotonic() + max_duration
        marker_pattern = re.compile(re.escape(marker) + r":(\d+)")
        rc = -1
        marker_seen_at = 0.0

        while time.monotonic() < deadline:
            buf = self._collector.tail_since(offset)
            matches = list(marker_pattern.finditer(buf))
            if matches:
                last = matches[-1]
                rc = int(last.group(1))
                if marker_seen_at <= 0.0:
                    marker_seen_at = time.monotonic()
            elif marker_seen_at <= 0.0:
                time.sleep(poll_interval)
                continue

            if marker_seen_at > 0.0 and (time.monotonic() - marker_seen_at) >= quiet_seconds:
                cleaned = self._strip_hv_m_marker(buf, marker)
                return cleaned, rc

            time.sleep(poll_interval)

        raise TerminalTimeoutError(
            f"timed out waiting for command status marker {marker}:$?",
            partial_output=self._collector.tail_since(offset),
        )

    def _wait_quiet(
        self,
        offset: int,
        quiet_seconds: float,
        max_duration: float,
        poll_interval: float,
        *,
        context: str,
    ) -> str:
        start = time.monotonic()
        deadline = start + max_duration
        last_len = self._collector.offset()
        last_output_at = start

        while time.monotonic() < deadline:
            current_len = self._collector.offset()
            if current_len > last_len:
                last_len = current_len
                last_output_at = time.monotonic()
            elif (time.monotonic() - last_output_at) >= quiet_seconds:
                return self._collector.tail_since(offset)

            time.sleep(poll_interval)

        raise TerminalTimeoutError(
            f"timed out waiting for terminal quiet period after command: {context}",
            partial_output=self._collector.tail_since(offset),
        )

    def _ensure_open(self) -> None:
        if not self._opened:
            self.open()

    @staticmethod
    def _trim_after_marker(output: str, marker: str) -> str:
        idx = output.find(marker)
        if idx < 0:
            return output
        return output[:idx]

    @staticmethod
    def _strip_hv_m_marker(output: str, marker: str) -> str:
        pattern = re.compile(re.escape(marker) + r":\d+")
        matches = list(pattern.finditer(output))
        if not matches:
            return output
        return output[: matches[-1].start()]
