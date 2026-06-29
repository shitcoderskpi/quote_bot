## Installation
```bash
git clone --recursive https://github.com/shitcoderskpi/quote_bot.git
```

## Setting up the environment
```bash
cp infra/.env.example infra/.env
```
Then fill <ins>everything blank</ins>.

| Variable     | Description                                                                                                                                            |
|--------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| `BOT_TOKEN`  | Telegram bot token from [@BotFather](https://t.me/BotFather).                                                                                          |
| `REDIS_HOST` | Hostname or IP of the Redis instance used as the job queue.                                                                                            |
| `TEMPLATE`   | Absolute path to the directory containing `.tem` template files inside the generator container. Defaults to `/app/templates` in the Docker image.      |
| `DPI`        | Output image resolution. Telegram displays stickers at 96 DPI, so higher values produce a sharper image that Telegram then downscales. Default: `300`. |
| `LOG_PATH`   | Log file directory, relative to each service's working directory (`services/{LOG_PATH}`).                                                              |

**P.s:** *Ask if you don't know...*

## Generator Service
The generator is a Rust service (`services/generator`) that receives jobs from a Redis queue, renders quote images via GPU (wgpu + vello), and pushes results back.

### Template format
Visual appearance is controlled by Minijinja templates in `services/generator/templates/`:

| File        | Theme           |
|-------------|-----------------|
| `light.tem` | Light (default) |
| `dark.tem`  | Dark            |

Each template is a series of COFFIN segments (SVG background + text layers) with Jinja expressions for dynamic values such as dimensions, content, username, and user role colors. The format is documented in [`services/generator/FORMAT.md`](services/generator/FORMAT.md).