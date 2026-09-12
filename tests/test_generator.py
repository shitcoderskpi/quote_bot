#!/usr/bin/env python3

import argparse
import json
import os
import sys
import time
from base64 import b64encode, b64decode
from dataclasses import dataclass, field
from io import BytesIO
from typing import Optional

import redis
import zstandard as zstd

REDIS_HOST = os.getenv("REDIS_HOST", "127.0.0.1")
REDIS_PORT = int(os.getenv("REDIS_PORT", "6379"))
JOBS_QUEUE = "generate:jobs"
RESULTS_QUEUE = "generate:results"

compressor = zstd.ZstdCompressor(level=9)
decompressor = zstd.ZstdDecompressor()


@dataclass
class FakeMessage:
    grad_id: int
    username: str
    user_status: Optional[str]
    user_role: Optional[str]
    content: str
    entities: list = field(default_factory=list)
    image_bytes: bytes = b""
    header: dict = field(default_factory=dict)
    dpi: Optional[int] = None
    theme: str = "light"

    def to_json_bytes(self) -> bytes:
        data = {
            "header": self.header,
            "grad_id": self.grad_id,
            "username": self.username,
            "user_status": self.user_status,
            "user_role": self.user_role,
            "content": self.content,
            "entities": self.entities,
            "image": None,
        }
        if self.dpi is not None:
            data["dpi"] = self.dpi
        if self.theme != "light":
            data["theme"] = self.theme
        return json.dumps(data, ensure_ascii=False).encode("utf-8")


def make_header(chat_id: int = 123456789, message_id: int = 42) -> dict:
    return {"message_id": message_id, "chat": {"id": chat_id}}


TESTS: dict[str, FakeMessage] = {
    "basic": FakeMessage(
        grad_id=3,
        username="Test User",
        user_status=None,
        user_role="member",
        content="Hello, this is a test message!",
        header=make_header(),
        image_bytes=None,
    ),
    "dark_theme": FakeMessage(
        grad_id=5,
        username="Dark Mode Fan",
        user_status="Premium",
        user_role="administrator",
        content="Testing dark theme rendering",
        header=make_header(chat_id=111111111),
        image_bytes=None,
        theme="dark",
    ),
    "custom_dpi": FakeMessage(
        grad_id=1,
        username="Hi-DPI User",
        user_status=None,
        user_role="member",
        content="Rendering at 150 DPI",
        header=make_header(),
        image_bytes=None,
        dpi=150,
    ),
    "with_entities": FakeMessage(
        grad_id=2,
        username="Entity User",
        user_status=None,
        user_role="member",
        content="Hello bold and italic world",
        entities=[
            {"type": "bold", "offset": 6, "length": 4},
            {"type": "italic", "offset": 15, "length": 6},
        ],
        header=make_header(),
        image_bytes=None,
    ),
    "long_text": FakeMessage(
        grad_id=6,
        username="Verbose User",
        user_status="jkbkjkijbknbkj",
        user_role="member",
        content=(
            "This is a much longer message to test word wrapping and layout. "
            "The generator should handle multi-line text gracefully and produce "
            "a properly sized output image. Let's see how it does with a few "
            "more sentences to really push the wrapping logic. 🚀✨🔥"
        ),
        header=make_header(),
        image_bytes=None,
    ),
    "no_avatar": FakeMessage(
        grad_id=4,
        username="No Avatar User",
        user_status=None,
        user_role="member",
        content="User without a profile photo",
        header=make_header(),
        image_bytes=b"",
    ),
}


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    DIM = "\033[2m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def run_test(
    r: redis.Redis,
    name: str,
    msg: FakeMessage,
    timeout: int,
    save_images: bool,
    output_dir: str,
) -> bool:
    print(f"\n{'-'* 60}")
    print(f"  {Colors.BOLD}{Colors.CYAN}TEST: {name}{Colors.RESET}")
    print(f"{'-' * 60}")
    print(f"  username : {msg.username}")
    print(f"  content  : {msg.content[:60]}{'...' if len(msg.content) > 60 else ''}")
    print(f"  theme    : {msg.theme}")
    print(f"  dpi      : {msg.dpi or 'default'}")
    print(f"  avatar   : {'yes' if msg.image_bytes else 'no'} ({0} bytes)")
    print(f"  entities : {len(msg.entities)}")

    payload = msg.to_json_bytes()
    compressed = compressor.compress(payload)

    print(
        f"\n  {Colors.DIM}Payload   : {len(payload)} bytes JSON -> {len(compressed)} bytes zstd{Colors.RESET}"
    )
    r.delete(RESULTS_QUEUE)

    t0 = time.monotonic()
    r.lpush(JOBS_QUEUE, compressed)
    print(f"  {Colors.DIM}Pushed to : {JOBS_QUEUE}{Colors.RESET}")

    print(
        f"  {Colors.DIM}Waiting   : (timeout {timeout}s) ...{Colors.RESET}",
        end="",
        flush=True,
    )
    result = r.brpop([RESULTS_QUEUE], timeout=timeout)
    elapsed_ms = (time.monotonic() - t0) * 1000

    if result is None:
        print(f"\r  {Colors.RED}TIMEOUT - no response in {timeout}s{Colors.RESET}")
        return False

    _, raw_result = result
    print(
        f"\r  {Colors.DIM}Got result: {len(raw_result)} bytes in {elapsed_ms:.0f} ms{Colors.RESET}    "
    )

    try:
        decompressed = decompressor.decompress(raw_result)
        result_json = json.loads(decompressed)
    except Exception as e:
        print(f"  {Colors.RED}DECOMPRESS/PARSE FAILED: {e}{Colors.RESET}")
        return False

    if "image" not in result_json:
        print(f"  {Colors.RED}Missing 'image' key in response{Colors.RESET}")
        return False
    try:
        img_bytes = b64decode(result_json["image"])
    except Exception as e:
        print(f"  {Colors.RED}base64 decode failed: {e}{Colors.RESET}")
        return False

    resp_header = result_json.get("header", {})
    if msg.header:
        expected_chat_id = msg.header.get("chat", {}).get("id")
        actual_chat_id = resp_header.get("chat", {}).get("id")
        if expected_chat_id != actual_chat_id:
            print(
                f"  {Colors.YELLOW}Header chat_id mismatch: expected {expected_chat_id}, got {actual_chat_id}{Colors.RESET}"
            )

    is_webp = img_bytes[:4] == b"RIFF" and img_bytes[8:12] == b"WEBP"
    fmt_str = "WebP" if is_webp else "unknown"

    print(f"  {Colors.DIM}Image     : {len(img_bytes)} bytes ({fmt_str}){Colors.RESET}")
    print(f"  {Colors.DIM}Header    : {json.dumps(resp_header)}{Colors.RESET}")
    print(f"  {Colors.DIM}Roundtrip : {elapsed_ms:.0f} ms{Colors.RESET}")

    if not is_webp:
        print(f"  {Colors.YELLOW}Image doesn't have WebP magic bytes{Colors.RESET}")

    if save_images:
        os.makedirs(output_dir, exist_ok=True)
        out_path = os.path.join(output_dir, f"{name}.webp")
        with open(out_path, "wb") as f:
            f.write(img_bytes)
        print(f"  {Colors.DIM}Saved to  : {out_path}{Colors.RESET}")

    print(f"  {Colors.GREEN}PASSED{Colors.RESET} ({elapsed_ms:.0f} ms)")
    return True


def main():
    parser = argparse.ArgumentParser(description="Test generator service locally")
    parser.add_argument("--host", default=REDIS_HOST, help="Redis host")
    parser.add_argument("--port", type=int, default=REDIS_PORT, help="Redis port")
    parser.add_argument(
        "--timeout", type=int, default=10, help="Seconds to wait for generator response"
    )
    parser.add_argument(
        "--save-images", action="store_true", help="Save output .webp images"
    )
    parser.add_argument(
        "--output-dir", default="tests/output", help="Directory for saved images"
    )
    parser.add_argument(
        "--test", type=str, default=None, help="Run a single test by name"
    )
    parser.add_argument("--list", action="store_true", help="List available test names")
    args = parser.parse_args()

    if args.list:
        print("Available tests:")
        for name in TESTS:
            print(f"  - {name}")
        return

    print(f"{Colors.BOLD}Generator Service Test Harness{Colors.RESET}")
    print(f"Redis: {args.host}:{args.port}")

    try:
        r = redis.Redis(host=args.host, port=args.port, db=0)
        r.ping()
    except redis.ConnectionError as e:
        print(
            f"{Colors.RED}Cannot connect to Redis at {args.host}:{args.port}: {e}{Colors.RESET}"
        )
        sys.exit(1)

    print(f"{Colors.GREEN}Redis connected{Colors.RESET}")

    if args.test:
        if args.test not in TESTS:
            print(
                f"{Colors.RED}Unknown test '{args.test}'. Use --list to see available tests.{Colors.RESET}"
            )
            sys.exit(1)
        tests_to_run = {args.test: TESTS[args.test]}
    else:
        tests_to_run = TESTS

    passed = 0
    failed = 0
    for name, msg in tests_to_run.items():
        ok = run_test(r, name, msg, args.timeout, args.save_images, args.output_dir)
        if ok:
            passed += 1
        else:
            failed += 1

    print(f"\n{'=' * 60}")
    total = passed + failed
    color = Colors.GREEN if failed == 0 else Colors.RED
    print(f"  {color}{passed}/{total} passed{Colors.RESET}", end="")
    if failed:
        print(f"  {Colors.RED}({failed} failed){Colors.RESET}", end="")
    print()
    print(f"{'=' * 60}")

    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
