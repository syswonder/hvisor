#!/usr/bin/env python3
"""Unified terminal helpers for QEMU UNIX socket and physical serial ports."""

from __future__ import annotations

import select
import socket
import time
import uuid
import re
from abc import ABC, abstractmethod
from dataclasses import dataclass

import serial


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


class Terminal:
    """High level terminal wrapper with command helpers."""

    def __init__(self, backend: TerminalBackend, encoding: str = "utf-8") -> None:
        self.backend = backend
        self.encoding = encoding
        self._opened = False

    @classmethod
    def from_qemu_socket(
        cls,
        path: str,
        connect_timeout: float = 10.0,
        io_timeout: float = 0.2,
        encoding: str = "utf-8",
    ) -> "Terminal":
        return cls(
            QemuSocketBackend(path=path, connect_timeout=connect_timeout, io_timeout=io_timeout),
            encoding=encoding,
        )

    @classmethod
    def from_serial(
        cls,
        port: str,
        baudrate: int = 115200,
        timeout: float = 0.2,
        encoding: str = "utf-8",
    ) -> "Terminal":
        return cls(SerialBackend(port=port, baudrate=baudrate, timeout=timeout), encoding=encoding)

    def open(self) -> None:
        if self._opened:
            return
        self.backend.open()
        self._opened = True

    def close(self) -> None:
        if not self._opened:
            return
        self.backend.close()
        self._opened = False

    def __enter__(self) -> "Terminal":
        self.open()
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

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
        deadline = time.monotonic() + duration
        chunks: list[str] = []
        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if chunk:
                chunks.append(chunk.decode(self.encoding, errors="replace"))
                continue
            time.sleep(poll_interval)
        return "".join(chunks)

    def send_until_get(
        self,
        command: str,
        timeout: float = 30.0,
        poll_interval: float = 0.05,
        include_marker_line: bool = False,
    ) -> str:
        self._ensure_open()
        marker = f"__HV_TERMINAL_DONE_{uuid.uuid4().hex}__"
        self.send(f"{command}; echo {marker}")

        deadline = time.monotonic() + timeout
        buf = ""
        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if chunk:
                buf += chunk.decode(self.encoding, errors="replace")
                if marker in buf:
                    if include_marker_line:
                        return buf
                    return self._trim_after_marker(buf, marker)
            time.sleep(poll_interval)
        raise TerminalTimeoutError(f"timed out waiting for terminal marker: {marker}")

    def send_until_quiet(
        self,
        command: str,
        quiet_seconds: float = 1.0,
        max_duration: float = 30.0,
        poll_interval: float = 0.05,
    ) -> str:
        self._ensure_open()
        self.send(command)

        start = time.monotonic()
        deadline = start + max_duration
        last_output_at = start
        buf = ""

        while True:
            now = time.monotonic()
            if now >= deadline:
                raise TerminalTimeoutError(
                    f"timed out waiting for terminal quiet period after command: {command}"
                )

            chunk = self.backend.read()
            if chunk:
                buf += chunk.decode(self.encoding, errors="replace")
                last_output_at = time.monotonic()
                continue

            if (now - last_output_at) >= quiet_seconds:
                return buf
            time.sleep(poll_interval)

    def send_and_drain(
        self,
        command: str,
        read_duration: float = 0.5,
        poll_interval: float = 0.05,
    ) -> str:
        """Send command and collect best-effort output for a fixed duration."""
        self._ensure_open()
        self.send(command)

        deadline = time.monotonic() + read_duration
        buf = ""
        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if chunk:
                buf += chunk.decode(self.encoding, errors="replace")
                continue
            time.sleep(poll_interval)
        return buf

    def read_until_quiet(
        self,
        quiet_seconds: float = 3.0,
        max_duration: float = 120.0,
        poll_interval: float = 0.05,
    ) -> str:
        """Continuously read until quiet for x seconds or total timeout."""
        self._ensure_open()
        start = time.monotonic()
        deadline = start + max_duration
        last_output_at = start
        buf = ""

        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if chunk:
                buf += chunk.decode(self.encoding, errors="replace")
                last_output_at = time.monotonic()
                continue

            now = time.monotonic()
            if (now - last_output_at) >= quiet_seconds:
                return buf
            time.sleep(poll_interval)
        return buf

    def run_until_quiet_with_status(
        self,
        command: str,
        quiet_seconds: float = 1.0,
        max_duration: float = 30.0,
        poll_interval: float = 0.05,
    ) -> tuple[str, int]:
        marker = f"__HV_TERMINAL_RC_{uuid.uuid4().hex}__"
        wrapped = f"{command}; echo {marker}0"
        self._ensure_open()
        self.send(wrapped)

        deadline = time.monotonic() + max_duration
        buf = ""
        marker_pattern = re.compile(re.escape(marker) + r"(\d+)")
        rc = -1
        marker_seen_at = 0.0

        while time.monotonic() < deadline:
            chunk = self.backend.read()
            if chunk:
                buf += chunk.decode(self.encoding, errors="replace")
                matches = list(marker_pattern.finditer(buf))
                if matches:
                    last = matches[-1]
                    rc = int(last.group(1))
                    if marker_seen_at <= 0.0:
                        marker_seen_at = time.monotonic()
                continue

            if marker_seen_at > 0.0 and (time.monotonic() - marker_seen_at) >= quiet_seconds:
                cleaned = self._strip_status_marker(buf, marker)
                return cleaned, rc

            time.sleep(poll_interval)

        raise TerminalTimeoutError(f"timed out waiting for command status marker: {marker}")

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
    def _extract_status_marker(output: str, marker: str) -> int:
        pattern = re.compile(re.escape(marker) + r"(\d+)")
        matches = list(pattern.finditer(output))
        if not matches:
            raise TerminalTimeoutError(f"status marker without exit code: {marker}")
        return int(matches[-1].group(1))

    @staticmethod
    def _strip_status_marker(output: str, marker: str) -> str:
        pattern = re.compile(re.escape(marker) + r"\d+")
        matches = list(pattern.finditer(output))
        if not matches:
            return output
        return output[: matches[-1].start()]
