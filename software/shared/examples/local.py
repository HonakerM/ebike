from dataclasses import dataclass, field
from datetime import datetime
import subprocess
import threading
import time
import base64
import orjson
import re

ECU_REGEX = re.compile(r"^EcuMessage.*raw_val\: *([0-9]*).([0-9]*).*$")
CTRL_REGEX = re.compile(
    r"^ControlReqMessage.*raw_val\: *([0-9]*).([0-9]*).*raw_val\: *([0-9]*).([0-9]*).*$"
)

INCLUDE_DEBUGGING = True


def forward_debug(proc):
    """Reads the subprocess stdout and forwards it to the Python stdout."""
    for line in iter(proc.stdout.readline, b""):
        print(line.decode(), end="")


@dataclass
class ECUMesg:
    throttle: float
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class CTLMesg:
    throttle_req: float
    brake_req: float
    timestamp: datetime = field(default_factory=datetime.now)


CAN_MESSAGES = []


def capture_can(proc):
    """Reads the subprocess stdout and forwards it to the Python stdout."""
    for line in iter(proc.stderr.readline, b""):
        val = line.decode()
        if ecu_match := ECU_REGEX.match(val):
            CAN_MESSAGES.append(
                ECUMesg(float(f"{ecu_match.group(1)}.{ecu_match.group(2)}"))
            )
        if ecu_match := CTRL_REGEX.match(val):
            CAN_MESSAGES.append(
                CTLMesg(
                    float(f"{ecu_match.group(1)}.{ecu_match.group(2)}"),
                    float(f"{ecu_match.group(3)}.{ecu_match.group(4)}"),
                )
            )
        print(val)


@dataclass
class Message:
    id: int
    raw_data: bytes

    def encode(self) -> str:
        self.raw_data = self.raw_data.ljust(8, bytes([0]))
        return f"{self.id}:{base64.b64encode(self.raw_data).decode()}"


def main():
    # Path to the compiled Rust example binary
    rust_binary = "./target/debug/examples/local.exe"  # Adjust the path if necessary

    # Start the Rust subprocess
    process = subprocess.Popen(
        rust_binary,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=1,  # Line-buffered
    )

    # Start a thread to forward the Rust process's stdout
    if INCLUDE_DEBUGGING:
        threading.Thread(target=forward_debug, args=(process,), daemon=True).start()
    threading.Thread(target=capture_can, args=(process,), daemon=True).start()

    try:
        # Send messages to the Rust process's stdin
        messages = [
            Message(2, bytes([128])),
            Message(2, bytes([255])),
            Message(2, bytes([0, 255])),
        ]
        for msg in messages:
            if process.poll():
                print("Rust process has terminated.")
                return

            print(f"Sending: {msg}")
            process.stdin.write((msg.encode() + "\n").encode())
            process.stdin.flush()
            time.sleep(5)  # Wait a bit before sending the next message

        data = open("./logs.txt", "w")
        for msg in CAN_MESSAGES:
            data.write(orjson.dumps(msg).decode())
            data.write("\n")

    finally:
        # Terminate the Rust process
        process.terminate()
        process.wait()


if __name__ == "__main__":
    main()
