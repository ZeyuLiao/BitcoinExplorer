# Variables
PROJECT_NAME = bitcoin-ingestion
PROJECT_VERSION = $(VERSION)
DOCKER_IMAGE_NAME = $(PROJECT_NAME):$(PROJECT_VERSION)
DOCKER_CONTAINER_NAME = $(PROJECT_NAME)-container
RPC_USER = $(RPC_USER)
RPC_PASSWORD = $(RPC_PASSWORD)
# get rpc url from LAN ip
#RPC_URL = http://$(shell ifconfig eth0 | sed -En 's/.*inet (addr:)?(([0-9]*\.){3}[0-9]*).*/\2/p'):8332
# set LAN ip for btc-core container
RPC_URL = 172.18.0.10

# Build rust project with PHONY
.PHONY: build
build:
	cd ./Ingestion && cargo build --release

# Build docker image
.PHONY: docker-build
docker-build:
	cd ./Ingestion && docker build -t $(DOCKER_IMAGE_NAME) .

# Run docker container
.PHONY: docker-run
docker-run:
	docker run \
	--rm \
	--network bitExp-network \
	--name $(DOCKER_CONTAINER_NAME) \
	$(DOCKER_IMAGE_NAME) \
	--user $(RPC_USER) \
	--pwd $(RPC_PASSWORD) \
	--rpc-url $(RPC_URL)

# Run bitcoin-core container
.PHONY: bitcoin-core
bitcoin-core:
	docker run -d \
	--name=bitcoin \
	--network=bitExp-network \
	--ip=$(RPC_URL) \
	-v /home/zoey/.bitcoin/bitcoin.conf:/home/bitcoin/.bitcoin/bitcoin.conf \
	-p 8332:8332 \
	-p 8333:8333 \
	ruimarinho/bitcoin-core:latest

# Release by tag version and y/n to confirm
.PHONY: release
release:
	git tag -a $(PROJECT_VERSION) -m "Release version $(PROJECT_VERSION)"
	git push origin $(PROJECT_VERSION)