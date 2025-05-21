.PHONY: build-penumbra-cli
build-penumbra-cli:
	(cd crates/penumbra ; cargo build --bin penumbra-disclosure-cli ; cp target/debug/penumbra-disclosure-cli ../..)

.PHONY: build-penumbra-cli-release
build-penumbra-cli-release:
	(cd crates/penumbra ; cargo build --bin penumbra-disclosure-cli --release; cp target/release/penumbra-disclosure-cli ../..)

.PHONY: build-penumbra-docker-release
build-penumbra-docker-release:
	DOCKER_BUILDKIT=1 docker \
		build \
		--build-arg BUILDKIT_INLINE_CACHE=1 \
		-t penumbra-disclosure-cli:latest \
		-f Dockerfile.penumbra \
		.