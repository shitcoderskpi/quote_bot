# Proto generation

From repo root:

```bash
protoc --go_out=services/bot --go_opt=module=bot proto/quote.proto
```

From `services/bot`:

```bash
protoc --go_out=. --go_opt=module=bot ../../proto/quote.proto
```

Requires:

```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
```

`protoc` must also be installed and available in `PATH`.
