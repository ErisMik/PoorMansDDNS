#!/bin/bash

# Need to have experimental CLI and Engine (buildx + buildkit)
export DOCKER_CLI_EXPERIMENTAL=enabled
export DOCKER_BUILDKIT=enabled

# Add QEMU stuff
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Create and bootstrap builder
docker buildx create --name mubuilder
docker buildx use mubuilder
docker buildx inspect --bootstrap

# Build and push images
docker buildx build . -t itseris/matt --push --platform "linux/amd64,linux/arm64"

# Cleanup
docker buildx rm mubuilder
