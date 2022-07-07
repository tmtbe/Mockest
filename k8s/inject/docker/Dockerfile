FROM --platform=$TARGETPLATFORM scratch
WORKDIR /app
COPY inject .
EXPOSE 80
ENTRYPOINT ["./inject"]