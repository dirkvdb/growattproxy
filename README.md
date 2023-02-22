# growattproxy

### Build docker image
```
docker build -t dirkvdb/growattproxy -f docker/BuildDockerfile .
```

# Build synology binary
```
cross.exe build --target x86_64-unknown-linux-gnu --release --features=sniffer
```