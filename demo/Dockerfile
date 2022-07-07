FROM --platform=$TARGETPLATFORM golang:1.17-alpine as builder
WORKDIR /app
COPY . .
RUN go mod tidy \
    && CGO_ENABLED=0 GOOS=linux go build -o ./demo

FROM --platform=$TARGETPLATFORM scratch
WORKDIR /app
COPY --from=builder /app/demo .
EXPOSE 80
ENTRYPOINT ["./demo"]