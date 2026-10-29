# Quotehuna

Telegram bot that renders quoted messages as stickers.

## Installation

```bash
git clone --recursive https://github.com/shitcoderskpi/quote_bot.git
cd quote_bot
```

## Configuration

```bash
cp infra/.env.example infra/.env
```

Then fill in everything that is blank.

| Variable            | Description                                                                                                                                             |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| `TELEGRAM_API_ID`   | `api_id` from my.telegram.org. Used by the local Bot API server.                                                                                        |
| `TELEGRAM_API_HASH` | `api_hash` from my.telegram.org.                                                                                                                        |
| `BOT_TOKEN`         | Bot token from @BotFather.                                                                                                                              |
| `TEMPLATE`          | Directory with the `.scm` templates inside the generator container. Default: `/app/templates`.                                                          |
| `DPI`               | Default output resolution. Telegram displays stickers at 96 DPI, so higher values give a sharper image that Telegram downscales. Default: `300`.        |
| `BOT_API_URL`       | Bot API endpoint. With the bundled compose file this is `http://telegram-bot-api:8081`. (optional)                                                      |
| `LOG_LVL`           | Bot log level (optional).                                                                                                                               |
| `LOG_PATH`          | Log directory, relative to each service's working directory (`services/{LOG_PATH}`) (optional).                                                         |

## Build and deployment

### 1. Build once the Bot API binary

The fork's own Dockerfile only copies a prebuilt binary, so build it first:

```bash
./infra/build-tba.sh            # auto-detects podman or docker
./infra/build-tba.sh docker     # or force a runtime
```

This compiles the sources in a throwaway container and writes
`extern/telegram-bot-api/binaries/<arch>/telegram-bot-api`.
Check the result:

```bash
file extern/telegram-bot-api/binaries/amd64/telegram-bot-api
```

Re-run the script only when the submodule is updated. The submodule itself is never modified.

### 2. Start the stack

```bash
podman-compose -f infra/docker-compose.yml up --build -d
# or
docker compose -f infra/docker-compose.yml up --build -d
```

## Usage

Reply to a message with:

| Command         | Result                                                         |
|-----------------|----------------------------------------------------------------|
| `/q`            | Quote the replied message                                      |
| `/q x3`         | Quote the replied message and the next two (up to 50)          |
| `/q dark`       | Dark theme (`light` is the default)                            |
| `/q 150`        | Render at 150 DPI (clamped to 72-300)                          |
| `/q x3 dark 150`| Options can be combined in any order                           |

Consecutive messages from one sender are grouped: the name is shown on the first bubble, the avatar and
tail on the last. Multi-message quotes need the local Bot API server, because they use `getMessages`.

## Generator service

The generator (`services/generator`) consumes jobs from NATS, renders quote images on the GPU and pushes
the results back.

### Template
Visual appearance is controlled by template files in services/generator/templates/:

| File        | Theme           |
|-------------|-----------------|
| light.scm   | Light (default) |
| dark.scm    | Dark            |

Templates use a Steel scheme to build a node layout tree with text, shapes, and gradients.
The script receives a payload variable with the quote details.
The full DSL reference is documented in [services/generator/DSL.md](services/generator/DSL.md).
