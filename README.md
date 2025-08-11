## Installation
```bash
git clone https://github.com/shitcoderskpi/quote_bot.git
git submodule --init --recursive
cd quote_bot
```
**### NOTE: Important**
```bash
sed -i '/shutdown(conn->client.sock, SHUTDOWN_WR);/c\
    if (conn->client.sock != INVALID_SOCKET) {\
        closesocket(conn->client.sock);\
        conn->client.sock = INVALID_SOCKET;\
    }' services/generator/extern/prometheus-cpp/3rdparty/civetweb/src/civetweb.c
```
That fixes issue when CivetWeb worker segfaults attempting to close socket
I'm not able to push this fix because it is in submodule
## Setting up the environment
```bash
cp infra/.env.example infra/.env
```
Then fill <ins>everything</ins>.

**P.s:** *Ask if don't know...*

**P.P.S.:** *Don't ask, i don't know ... (⌐■_■)*
