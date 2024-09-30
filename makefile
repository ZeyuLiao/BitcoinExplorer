# make file to build release rust project and build docker image use dockerfile

# Variables
PROJECT_NAME = bitcoin-ingestion
PROJECT_VERSION = $(VERSION)
DOCKER_IMAGE_NAME = $(PROJECT_NAME):$(PROJECT_VERSION)
DOCKER_CONTAINER_NAME = $(PROJECT_NAME)-container
RPC_USER = $(RPC_USER)
RPC_PASSWORD = $(RPC_PASSWORD)


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
	--network host \
	--name $(DOCKER_CONTAINER_NAME) \
	$(DOCKER_IMAGE_NAME) \
	--user $(RPC_USER) \
	--pwd $(RPC_PASSWORD)
