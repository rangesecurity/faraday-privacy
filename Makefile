.PHONY: test-validator
test-validator:
	solana-test-validator --bpf-program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb token2022.so --reset

.PHONY: build-penumbra-docker-release
build-penumbra-docker-release:
	DOCKER_BUILDKIT=1 docker \
		build \
		--build-arg BUILDKIT_INLINE_CACHE=1 \
		-t penumbra-disclosure-cli:latest \
		-f Dockerfile.penumbra \
		.