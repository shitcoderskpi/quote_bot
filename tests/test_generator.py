import argparse
import asyncio
from enum import member
import os
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from itertools import count
from typing import Optional

import nats
import zstandard as zstd
from nats.errors import TimeoutError as NatsTimeout
from nats.js.errors import BadRequestError

import quote_pb2

NATS_URL = os.getenv("NATS_URL", "nats://127.0.0.1:4222")
JOBS_QUEUE = "generate:jobs"
RESULTS_QUEUE = "generate:results"
PUBLISH_CHUNK = 500
PROBE_INTERVAL = 5.0
WEBP_MAGIC_RIFF = b"RIFF"
WEBP_MAGIC_TAG = b"WEBP"

compressor = zstd.ZstdCompressor(level=9)
decompressor = zstd.ZstdDecompressor()
message_ids = count(1)


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    DIM = "\033[2m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def paint(color: str, text: str) -> str:
    return f"{color}{text}{Colors.RESET}"


@dataclass
class FakeMessage:
    username: str
    content: str
    sender_index: int = 0
    color_index: int = 0
    user_status: Optional[str] = None
    user_role: Optional[str] = "member"
    entities: list = field(default_factory=list)
    image_bytes: Optional[bytes] = None
    message_id: int = field(default_factory=lambda: next(message_ids))

    def __post_init__(self):
        units = len(self.content.encode("utf-16-le")) // 2
        for ent in self.entities:
            if ent["offset"] < 0 or ent["offset"] + ent["length"] > units:
                raise ValueError(f"entity out of range: {ent} in {self.content!r}")

    def to_pb(self) -> "quote_pb2.QuoteMessage":
        msg = quote_pb2.QuoteMessage(message_id=self.message_id, content=self.content)
        author = msg.author
        author.sender_index = self.sender_index
        author.color_index = self.color_index
        author.username = self.username
        if self.user_status is not None:
            author.user_status = self.user_status
        if self.user_role is not None:
            author.user_role = self.user_role
        if self.image_bytes is not None:
            msg.image = self.image_bytes
        for ent in self.entities:
            pb_ent = msg.entities.add()
            pb_ent.type = ent["type"]
            pb_ent.offset = ent["offset"]
            pb_ent.length = ent["length"]
        return msg


@dataclass
class FakeQuote:
    messages: list[FakeMessage]
    chat_id: int = 123456789
    theme: str = "light"
    dpi: Optional[int] = None

    @property
    def message_id(self) -> int:
        return self.messages[0].message_id

    def to_protobuf_bytes(self) -> bytes:
        req = quote_pb2.QuoteRequest(chat_id=self.chat_id, theme=self.theme)
        if self.dpi is not None:
            req.dpi = self.dpi
        req.messages.extend(m.to_pb() for m in self.messages)
        return req.SerializeToString()

    def to_job(self) -> bytes:
        return compressor.compress(self.to_protobuf_bytes())


def solo(
    chat_id: int = 123456789, theme: str = "light", dpi: Optional[int] = None, **kw
) -> FakeQuote:
    return FakeQuote([FakeMessage(**kw)], chat_id=chat_id, theme=theme, dpi=dpi)


def build_tests() -> dict[str, FakeQuote]:
    return {
        "basic": solo(
            color_index=3,
            username="Test User",
            content="Hello, this is a test message!",
        ),
        "dark_theme": solo(
            color_index=5,
            username="Dark Mode Fan",
            user_status="Premium",
            user_role="administrator",
            content="Testing dark theme rendering",
            chat_id=111111111,
            theme="dark",
        ),
        "custom_dpi": solo(
            color_index=1,
            username="Hi-DPI User",
            content="Rendering at 150 DPI",
            dpi=150,
        ),
        "with_entities": solo(
            color_index=2,
            username="Entity User",
            content="Hello bold and italic world",
            entities=[
                {"type": "bold", "offset": 6, "length": 4},
                {"type": "italic", "offset": 15, "length": 6},
            ],
        ),
        "more_entities": solo(
            color_index=3,
            username="More Entities User",
            user_status="entities!",
            user_role="member",
            content="Spoiler text, code here, and underline!",
            entities=[
                {"type": "spoiler", "offset": 0, "length": 7},
                {"type": "code", "offset": 14, "length": 9},
                {"type": "underline", "offset": 29, "length": 9},
            ],
        ),
        "all_entities": solo(
            color_index=4,
            username="All Entities",
            content="b i u s code pre spoiler link mention emoji",
            entities=[
                {"type": "bold", "offset": 0, "length": 1},
                {"type": "italic", "offset": 2, "length": 1},
                {"type": "underline", "offset": 4, "length": 1},
                {"type": "strikethrough", "offset": 6, "length": 1},
                {"type": "code", "offset": 8, "length": 4},
                {"type": "pre", "offset": 13, "length": 3},
                {"type": "spoiler", "offset": 17, "length": 7},
                {"type": "text_link", "offset": 25, "length": 4},
                {"type": "mention", "offset": 30, "length": 7},
                {"type": "custom_emoji", "offset": 38, "length": 5},
            ],
        ),
        "long_text": solo(
            color_index=6,
            username="Verbose User",
            user_status="Long statusgvgv",
            content=(
                "This is a much longer message to test word wrapping and layout. "
                "The generator should handle multi-line text gracefully and produce "
                "a properly sized output image. Let's see how it does with a few "
                "more sentences to really push the wrapping logic. 🚀✨🔥"
            ),
        ),
        "no_avatar": solo(
            color_index=4,
            username="No Avatar User",
            content="User without a profile photo",
            image_bytes=b"",
        ),
        "multi_same_sender": FakeQuote(
            [
                FakeMessage("Alice Smith", "First message in the run", color_index=2),
                FakeMessage("Alice Smith", "Second message", color_index=2),
                FakeMessage("Alice Smith", "Third and last message", color_index=2),
            ]
        ),
        "multi_two_groups": FakeQuote(
            [
                FakeMessage("Alice Smith", "Hi!", sender_index=0, color_index=2),
                FakeMessage(
                    "Alice Smith", "Anyone around?", sender_index=0, color_index=2
                ),
                FakeMessage("Bob", "Yep, here", sender_index=1, color_index=5),
                FakeMessage("Bob", "What's up?", sender_index=1, color_index=5),
            ]
        ),
        "multi_alternating": FakeQuote(
            [
                FakeMessage("Alice Smith", "Question?", sender_index=0, color_index=2),
                FakeMessage("Bob", "Answer.", sender_index=1, color_index=5),
                FakeMessage("Alice Smith", "Thanks!", sender_index=0, color_index=2),
            ]
        ),
        "multi_entities": FakeQuote(
            [
                FakeMessage("Entity User", "plain text, no entities", color_index=1),
                FakeMessage(
                    "Entity User",
                    "Hello bold and italic world",
                    color_index=1,
                    entities=[
                        {"type": "bold", "offset": 6, "length": 4},
                        {"type": "italic", "offset": 15, "length": 6},
                    ],
                ),
                FakeMessage(
                    "Entity User",
                    "code here and a link",
                    color_index=1,
                    entities=[
                        {"type": "code", "offset": 0, "length": 4},
                        {"type": "text_link", "offset": 16, "length": 4},
                    ],
                ),
            ]
        ),
        "multi_roles": FakeQuote(
            [
                FakeMessage(
                    "Admin Person",
                    "Reminder: be nice",
                    sender_index=0,
                    color_index=3,
                    user_status="Moderator",
                    user_role="administrator",
                ),
                FakeMessage(
                    "Admin Person",
                    "Rules are pinned",
                    sender_index=0,
                    color_index=3,
                    user_status="Moderator",
                    user_role="administrator",
                ),
                FakeMessage(
                    "The Creator",
                    "Thanks all",
                    sender_index=1,
                    color_index=0,
                    user_status="Owner",
                    user_role="creator",
                ),
            ],
            theme="dark",
        ),
        "multi_avatar": FakeQuote(
            [
                FakeMessage("Pic User", "Message one", sender_index=0),
                FakeMessage("Pic User", "Message two", sender_index=0),
                FakeMessage("No Pic", "I have initials", sender_index=1, color_index=4),
            ]
        ),
        "multi_long": FakeQuote(
            [
                FakeMessage("Verbose User", f"Message number {i}", color_index=6)
                for i in range(5)
            ]
            + [
                FakeMessage(
                    "Verbose User",
                    "This is a much longer message to test word wrapping inside a group. "
                    * 3,
                    color_index=6,
                )
            ]
        ),
        "multi_hidpi": FakeQuote(
            [
                FakeMessage(
                    "Alice Smith", "High DPI, first", sender_index=0, color_index=2
                ),
                FakeMessage("Bob", "High DPI, second", sender_index=1, color_index=5),
            ],
            dpi=150,
        ),
    }


def is_webp(data: bytes) -> bool:
    return data[:4] == WEBP_MAGIC_RIFF and data[8:12] == WEBP_MAGIC_TAG


def decode_result(raw: bytes) -> "quote_pb2.ResultImage":
    result = quote_pb2.ResultImage()
    result.ParseFromString(decompressor.decompress(raw))
    return result


async def await_result(sub, message_id: int, deadline: float):
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise NatsTimeout()
        raw = (await sub.next_msg(timeout=remaining)).data
        try:
            result = decode_result(raw)
        except Exception:
            return raw, None
        if result.message_id == message_id:
            return raw, result


async def run_test(
    nc, js, name: str, msg: FakeMessage, timeout: float, output_dir: Optional[str]
) -> bool:
    print(f"\n{'-' * 60}")
    print(f"  {Colors.BOLD}{Colors.CYAN}TEST: {name}{Colors.RESET}")
    print(f"{'-' * 60}")
    print(f"  theme    : {msg.theme}")
    print(f"  dpi      : {msg.dpi or 'default'}")
    senders = len({m.sender_index for m in msg.messages})
    print(
        f"  messages : {len(msg.messages)} ({senders} sender{'s' if senders != 1 else ''})"
    )
    for i, m in enumerate(msg.messages):
        avatar = f"{len(m.image_bytes)}B" if m.image_bytes else "no"
        print(
            f"  [{i}] s{m.sender_index} {m.username}: {m.content[:40]!r}"
            f"{'...' if len(m.content) > 40 else ''} (avatar {avatar}, {len(m.entities)} entities)"
        )

    payload = msg.to_protobuf_bytes()
    compressed = compressor.compress(payload)
    print(
        f"\n  {paint(Colors.DIM, f'Payload   : {len(payload)} bytes PB -> {len(compressed)} bytes zstd')}"
    )

    sub = await nc.subscribe(RESULTS_QUEUE)
    try:
        t0 = time.monotonic()
        await js.publish(JOBS_QUEUE, compressed)
        print(f"  {paint(Colors.DIM, f'Pushed to : {JOBS_QUEUE}')}")
        print(
            f"  {paint(Colors.DIM, f'Waiting   : (timeout {timeout}s) ...')}",
            end="",
            flush=True,
        )

        try:
            raw, result = await await_result(sub, msg.message_id, t0 + timeout)
        except NatsTimeout:
            print(f"\r  {paint(Colors.RED, f'TIMEOUT - no response in {timeout}s')}")
            return False
        elapsed_ms = (time.monotonic() - t0) * 1000
    finally:
        await sub.unsubscribe()

    print(
        f"\r  {paint(Colors.DIM, f'Got result: {len(raw)} bytes in {elapsed_ms:.0f} ms')}    "
    )

    if result is None:
        print(f"  {paint(Colors.RED, 'DECOMPRESS/PARSE FAILED')}")
        return False

    if not result.image:
        print(f"  {paint(Colors.RED, 'Missing image in response')}")
        return False

    img = result.image
    ok = True

    if result.chat_id != msg.chat_id:
        print(
            f"  {paint(Colors.RED, f'chat_id mismatch: expected {msg.chat_id}, got {result.chat_id}')}"
        )
        ok = False

    webp = is_webp(img)
    print(
        f"  {paint(Colors.DIM, f'Image     : {len(img)} bytes ({'WebP' if webp else 'unknown'})')}"
    )
    print(
        f"  {paint(Colors.DIM, f'Header    : chat_id={result.chat_id} message_id={result.message_id}')}"
    )
    print(f"  {paint(Colors.DIM, f'Roundtrip : {elapsed_ms:.0f} ms')}")

    if not webp:
        print(f"  {paint(Colors.RED, 'Image does not have WebP magic bytes')}")
        ok = False

    if output_dir:
        os.makedirs(output_dir, exist_ok=True)
        out_path = os.path.join(output_dir, f"{name}.webp")
        with open(out_path, "wb") as f:
            f.write(img)
        print(f"  {paint(Colors.DIM, f'Saved to  : {out_path}')}")

    if ok:
        print(f"  {paint(Colors.GREEN, 'PASSED')} ({elapsed_ms:.0f} ms)")
    else:
        print(f"  {paint(Colors.RED, 'FAILED')}")
    return ok


async def run_load_test(nc, js, total: int, idle_timeout: float) -> bool:
    print(
        f"\n{paint(Colors.BOLD + Colors.CYAN, f'Starting Load Test: {total} messages')}"
    )

    received = 0
    last_activity = time.monotonic()
    finished_at = 0.0
    done = asyncio.Event()

    async def on_result(_):
        nonlocal received, last_activity, finished_at
        received += 1
        last_activity = time.monotonic()
        if received % 1000 == 0:
            print(f"  Received {received} / {total} results...")
        if received >= total:
            finished_at = last_activity
            done.set()

    sub = await nc.subscribe(RESULTS_QUEUE, cb=on_result)
    job = solo(
        color_index=4,
        username="All Entities",
        content="b i u s code pre spoiler link mention emoji",
        entities=[
            {"type": "bold", "offset": 0, "length": 1},
            {"type": "italic", "offset": 2, "length": 1},
            {"type": "underline", "offset": 4, "length": 1},
            {"type": "strikethrough", "offset": 6, "length": 1},
            {"type": "code", "offset": 8, "length": 4},
            {"type": "pre", "offset": 13, "length": 3},
            {"type": "spoiler", "offset": 17, "length": 7},
            {"type": "text_link", "offset": 25, "length": 4},
            {"type": "mention", "offset": 30, "length": 7},
            {"type": "custom_emoji", "offset": 38, "length": 5},
        ],
    ).to_job()

    try:
        print(f"Publishing {total} jobs to '{JOBS_QUEUE}'...")
        start = time.monotonic()
        published = 0
        while published < total:
            batch = min(PUBLISH_CHUNK, total - published)
            await asyncio.gather(*(js.publish(JOBS_QUEUE, job) for _ in range(batch)))
            published += batch
            if published % 1000 < PUBLISH_CHUNK:
                print(f"  Published {published} / {total}...")

        print(
            paint(
                Colors.YELLOW,
                "All jobs published. Waiting for generator to finish processing...",
            )
        )
        last_activity = time.monotonic()
        while not done.is_set() and time.monotonic() - last_activity < idle_timeout:
            try:
                await asyncio.wait_for(done.wait(), timeout=0.5)
            except asyncio.TimeoutError:
                pass
    finally:
        await sub.unsubscribe()

    if received < total:
        print(
            paint(
                Colors.RED, f"Timeout waiting for results! Received {received}/{total}"
            )
        )
        return False

    elapsed = finished_at - start
    print(
        paint(
            Colors.GREEN,
            f"Successfully processed {total} jobs in {elapsed:.2f} seconds ({total / elapsed:.1f} msg/s).",
        )
    )
    return True


def start_flamegraph() -> subprocess.Popen:
    env = os.environ.copy()
    env.setdefault("TEMPLATE", "templates")
    print(
        paint(
            Colors.YELLOW,
            "Starting cargo flamegraph... (this may take a moment to build)",
        )
    )
    return subprocess.Popen(
        ["cargo", "flamegraph"],
        cwd="services/generator",
        env=env,
        start_new_session=True,
    )


def signal_group(proc: subprocess.Popen, sig: int):
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


async def stop_flamegraph(proc: subprocess.Popen, grace: float = 60.0):
    print(paint(Colors.YELLOW, "Stopping cargo flamegraph to generate SVG..."))
    signal_group(proc, signal.SIGINT)
    try:
        await asyncio.wait_for(asyncio.to_thread(proc.wait), timeout=grace)
    except asyncio.TimeoutError:
        print(paint(Colors.RED, "Flamegraph did not exit in time, terminating"))
        signal_group(proc, signal.SIGTERM)
        try:
            await asyncio.wait_for(asyncio.to_thread(proc.wait), timeout=10)
        except asyncio.TimeoutError:
            signal_group(proc, signal.SIGKILL)
            await asyncio.to_thread(proc.wait)
        return
    print(
        paint(Colors.GREEN, "Flamegraph generated at services/generator/flamegraph.svg")
    )


async def wait_until_ready(
    nc, js, proc: Optional[subprocess.Popen], timeout: float
) -> bool:
    print(
        paint(
            Colors.YELLOW, f"Waiting for generator to come up (up to {timeout:g}s)..."
        )
    )
    started = time.monotonic()
    deadline = started + timeout
    sub = await nc.subscribe(RESULTS_QUEUE)
    try:
        while True:
            if proc is not None and proc.poll() is not None:
                print(
                    paint(
                        Colors.RED,
                        f"Generator process exited early with code {proc.returncode}",
                    )
                )
                return False
            now = time.monotonic()
            if now >= deadline:
                print(paint(Colors.RED, "Timed out waiting for generator"))
                return False
            probe = solo(color_index=3, username="Probe", content="probe")
            await js.publish(JOBS_QUEUE, probe.to_job())
            try:
                await await_result(
                    sub, probe.message_id, min(now + PROBE_INTERVAL, deadline)
                )
                break
            except NatsTimeout:
                print(
                    f"\r  {paint(Colors.DIM, f'still waiting... {time.monotonic() - started:.0f}s')}",
                    end="",
                    flush=True,
                )
        while True:
            try:
                await sub.next_msg(timeout=1.0)
            except NatsTimeout:
                break
    finally:
        await sub.unsubscribe()
    print(
        f"\r{paint(Colors.GREEN, f'Generator ready after {time.monotonic() - started:.0f}s')}          "
    )
    return True


async def ensure_streams(js):
    for queue in (JOBS_QUEUE, RESULTS_QUEUE):
        try:
            await js.add_stream(name=queue, subjects=[queue])
        except BadRequestError:
            pass


def parse_args():
    parser = argparse.ArgumentParser(description="Test generator service locally")
    parser.add_argument("--nats-url", default=NATS_URL)
    parser.add_argument("--timeout", type=float, default=10)
    parser.add_argument("--save-images", action="store_true")
    parser.add_argument("--output-dir", default="tests/output")
    parser.add_argument("--test", default=None)
    parser.add_argument("--load", type=int, default=0)
    parser.add_argument("--startup-timeout", type=float, default=900)
    parser.add_argument("--flamegraph", action="store_true")
    parser.add_argument("--list", action="store_true")
    return parser.parse_args()


async def main() -> int:
    args = parse_args()
    tests = build_tests()

    if args.list:
        print("Available tests:")
        for name in tests:
            print(f"  - {name}")
        return 0

    if args.test and args.test not in tests:
        print(
            paint(
                Colors.RED,
                f"Unknown test '{args.test}'. Use --list to see available tests.",
            )
        )
        return 1

    print(paint(Colors.BOLD, "Generator Service Test Harness"))
    print(f"NATS: {args.nats_url}")

    try:
        nc = await nats.connect(args.nats_url)
        js = nc.jetstream()
        await ensure_streams(js)
    except Exception as e:
        print(paint(Colors.RED, f"Cannot connect to NATS at {args.nats_url}: {e}"))
        return 1

    print(paint(Colors.GREEN, "NATS connected"))

    try:
        if args.load > 0:
            generator_proc = None
            if args.flamegraph:
                generator_proc = start_flamegraph()
            try:
                if not await wait_until_ready(
                    nc, js, generator_proc, args.startup_timeout
                ):
                    return 1
                try:
                    await js.purge_stream(RESULTS_QUEUE)
                except Exception:
                    pass
                ok = await run_load_test(nc, js, args.load, args.timeout)
            finally:
                if generator_proc:
                    await stop_flamegraph(generator_proc)
            return 0 if ok else 1

        selected = {args.test: tests[args.test]} if args.test else tests
        output_dir = args.output_dir if args.save_images else None

        passed = failed = 0
        for name, msg in selected.items():
            if await run_test(nc, js, name, msg, args.timeout, output_dir):
                passed += 1
            else:
                failed += 1

        color = Colors.GREEN if failed == 0 else Colors.RED
        suffix = f"  {paint(Colors.RED, f'({failed} failed)')}" if failed else ""
        print(f"\n{'=' * 60}")
        print(f"  {paint(color, f'{passed}/{passed + failed} passed')}{suffix}")
        print("=" * 60)
        return 0 if failed == 0 else 1
    finally:
        await nc.close()


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
