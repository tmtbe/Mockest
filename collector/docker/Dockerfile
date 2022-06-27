FROM --platform=linux/arm64/v8 scratch
WORKDIR /app
COPY collector .
EXPOSE 80
ENTRYPOINT ["./collector"]