FROM golang:1.27-alpine3.23 AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o bot .

FROM alpine:3.23
RUN apk --no-cache add ca-certificates tzdata

WORKDIR /app
COPY --from=builder /app/bot .

CMD ["/app/bot"]
