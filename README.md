## Installation
```bash
git clone https://github.com/shitcoderskpi/quote_bot.git
cd quote_bot
git submodule update --init --recursive
```
## Setting up the environment
```bash
cp infra/.env.example infra/.env
```
Then fill <ins>everything</ins>.

**P.s:** *Ask if don't know...*

**P.P.S.:** *Don't ask, i don't know ... (⌐■_■)*

## Known Issues
### Generator service exits with Code 139 / Segfaults
The generator may segfault when CivetWeb workers attempt to close a socket. This can be fixed with the following patch from root of this repo (not from generator dir):
```bash
sed -i '/shutdown(conn->client.sock, SHUTDOWN_WR);/c\
if (conn->client.sock != INVALID_SOCKET) {\
    closesocket(conn->client.sock);\
    conn->client.sock = INVALID_SOCKET;\
}' services/generator/extern/prometheus-cpp/3rdparty/civetweb/src/civetweb.c
````

> ⚠️ Note: This fix modifies a submodule, so it cannot be pushed directly.
