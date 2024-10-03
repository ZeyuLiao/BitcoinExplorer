[![Build and Push Docker Image](https://github.com/ZeyuLiao/BitcoinExplorer/actions/workflows/makefile.yml/badge.svg)](https://github.com/ZeyuLiao/BitcoinExplorer/actions/workflows/makefile.yml)

# BitcoinExplorer
a bitcoin explorer to view on chain and off chain metrics

### Main features
Extract block information from bitcoin-core and upload the information to the rds mysql database

run binary will create cheduled tasks to extract data every miniutes
Binary file is located at [here](Ingestion/target/release/Ingestion) 

run binary with required rpc username and password
```
Usage: Ingestion [OPTIONS] --user <USER> --pwd <PWD>

Options:
      --user <USER>        The user name for the RPC server
      --pwd <PWD>          The password for the RPC server
      --rpc-url <RPC_URL>  The RPC server URL [default: http://127.0.0.1:8332]
  -h, --help               Print help
  -V, --version            Print version
```

### Build Process
`make build`: build binary file from rust project

`make docker-build VERSION={}`: build docker image

`make docker-run VERSION={} RPC_USER={} RPC_PASSWORD={}`: run container with rpc username and password

### Release Process:
#### Auto Release:
`make release VERSION={}`: this command will trigger github actions to build and tag with version and then push images to dockerhub
#### Mannual Release:
In order github action occurs any error, you can push image to dockerhub mannully

`make docker-push VERSION={}`: push local image to dockerhub

