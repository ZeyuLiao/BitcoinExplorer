# Variables
PROJECT_NAME=bitcoin-ingestion
DOCKERHUB=zoliao2024
PROJECT_VERSION=$(VERSION)
DOCKER_IMAGE_NAME=$(DOCKERHUB)/$(PROJECT_NAME):$(PROJECT_VERSION)
DOCKER_CONTAINER_NAME=$(PROJECT_NAME)-container
RPC_USER=$(RPC_USER)
RPC_PASSWORD=$(RPC_PASSWORD)
RPC_URL = ${RPC_URL}
#RPC_URL = 172.18.0.10
DB_USER=$(DB_USER)
DB_PASSWORD=$(DB_PASSWORD)
DB_HOST=$(DB_HOST)
DB_PORT=$(DB_PORT)
DB_NAME=$(DB_NAME)
# get rpc url from LAN ip
#RPC_URL = http://$(shell ifconfig eth0 | sed -En 's/.*inet (addr:)?(([0-9]*\.){3}[0-9]*).*/\2/p'):8332
# set LAN ip for btc-core container

# Build rust project with PHONY
.PHONY: build
build:
	cd ./Ingestion && cargo build --release

# Build docker image
.PHONY: docker-build
docker-build:
	cd ./Ingestion && docker build -t $(DOCKER_IMAGE_NAME) .

# Push docker image to docker hub
.PHONY: docker-push
docker-push:
	docker push $(DOCKER_IMAGE_NAME)

# Run docker container
.PHONY: docker-run
docker-run:
	docker run -d \
	--network bitExp-network \
	--name $(DOCKER_CONTAINER_NAME) \
	$(DOCKER_IMAGE_NAME) \
	--rpc-user $(RPC_USER) \
	--rpc-pwd $(RPC_PASSWORD) \
	--rpc-url $(RPC_URL) \
	--db-user $(DB_USER) \
	--db-pwd $(DB_PASSWORD) \
	--db-host $(DB_HOST)

# Clean docker container
.PHONY: docker-clean
docker-clean:
	docker stop $(DOCKER_CONTAINER_NAME)
	docker rm $(DOCKER_CONTAINER_NAME)

# Run bitcoin-core container
.PHONY: bitcoin-core
bitcoin-core:
	docker pull ruimarinho/bitcoin-core:latest
	docker run -d \
	--name=bitcoin \
	--network=bitExp-network \
	--ip=$(RPC_URL) \
	-v $(HOME)/.bitcoin/bitcoin.conf:/home/bitcoin/.bitcoin/bitcoin.conf \
	-p 8332:8332 \
	-p 8333:8333 \
	ruimarinho/bitcoin-core:latest

# Clean bitcoin-core container
.PHONY: bitcoin-core-clean
bitcoin-core-clean:
	docker stop bitcoin
	docker rm bitcoin

# Release by tag version and y/n to confirm
.PHONY: release
release:
	git tag -a $(PROJECT_VERSION) -m "Release version $(PROJECT_VERSION)"
	git push origin $(PROJECT_VERSION)