# disclosure-library

# Specification

OpenAPI is used to manage the API calls, response types, etc... For documentation about how to query the API you can take the `openapi.yaml` file and paste it into an editor like swagger (https://editor-next.swagger.io/)

# Docker Image

## Penumbra

To compile the docker image for disclosing penumbra transactions

```shell
$>  DOCKER_BUILDKIT=1 docker \
  build \
  --build-arg BUILDKIT_INLINE_CACHE=1 \
  -t penumbra-disclosure-cli:latest \
  -f Dockerfile.penumbra \
  .
```

You can run the API service with a docker compose file like the following

```yaml
services:
  penumbra:
    image: penumbra-disclosure-cli:latest
    command: --rpc-url http://penumbra_rpc.example.com:8080 api --listen-url 0.0.0.0:1337
    ports:
      - 1337:1337
```

You can then generate disclosure bundles from the CLI like so

```shell
$> curl -X POST \
    -H "Content-Type: application/json" \
    -d '{"fullViewingKey": "penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq", "transactionHash": "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf"}' \
    http://localhost:1337/disclose/transaction
{"disclosureTransactions":{"transactions":[{"transactionHash":"c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf","protocol":"penumbra","chainId":"penumbra-testnet-phobos-x3b26d34a","counterparties":[{"role":"receiver","address":"penumbra147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahh09cxmz","assets":[{"identifier":"wtest_usd","amount":"100000000000000000000","decimals":18}]},{"role":"sender","address":"penumbra1alp9a75s438d33rs5nt245ue2wctfne7x4c3v7afyslmwefltgpzm7r0jgmxphrcva6h44v9pe3esstnkw5fsha54rcp7xpmaphxx76scql92mefzg366ckwcy425s3y5657ll","assets":[{"identifier":"wtest_usd","amount":"100000000000000000000","decimals":18}]}],"timestamp":"1745289093","metadata":[{"transactionType":"Spend"}]}]}}   
```

# Development

## Penumbra

To test the penumbra disclosure client you'll need to setup a local environment

Clone the penumbra repository to setup the development environment

```shell
$> git clone https://github.com/penumbra-zone/penumbra
$> cd penumbra
$> git checkout v1.4.0
```

Setup the development environment

```shell
$> nix develop
$> just dev
```
