import subprocess
import threading
import time
import base64

def forward_output(process):
    """Reads the subprocess stdout and forwards it to the Python stdout."""
    for line in iter(process.stdout.readline, b""):
        print(line.decode(), end="")

def main():
    # Path to the compiled Rust example binary
    rust_binary = "./target/debug/examples/local.exe"  # Adjust the path if necessary

    # Start the Rust subprocess
    process = subprocess.Popen(
        rust_binary,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        bufsize=1,  # Line-buffered
    )

    # Start a thread to forward the Rust process's stdout
    threading.Thread(target=forward_output, args=(process,), daemon=True).start()

    try:
        # Send messages to the Rust process's stdin
        messages = [
            f"1:{base64.b64encode(bytes([128,0,0,0,0,0,0,0])).decode()}",
            f"1:{base64.b64encode(bytes([255,0,0,0,0,0,0,0])).decode()}",
        ]
        for msg in messages:
            print(f"Sending: {msg}")
            process.stdin.write((msg + "\n").encode())
            process.stdin.flush()
            time.sleep(5)  # Wait a bit before sending the next message

        # Allow some time for the Rust process to process messages
        time.sleep(10)

    finally:
        # Terminate the Rust process
        process.terminate()
        process.wait()

if __name__ == "__main__":
    main()