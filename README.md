# Disclosure Library

The disclosure library is used to create disclosure data bundles for Penumbra for auditing purposes that can be used to reveal information about transactions that a wallet has sent on the Penumbra network. The library includes a CLI, an sdk client, and an API that can be used to generate the disclosure bundles.

The API allows remote users to create disclosure data bundles without having to use an gRPC connection, while the CLI + sdk client can be used directly with an gRPC to generate these bundles.

# API Specification

OpenAPI is used to describe the data bundles, in addition to the API calls that can be used with a deployed version of the API. For documentation, copy the contents of `openapi.yaml` file and paste it into an editor like swagger (https://editor-next.swagger.io/).

## API Endpoints

The API provides two main endpoints for transaction disclosure:

### 1. Single Transaction Disclosure

```
POST /disclose/transaction
```

Discloses information about a single transaction.

**Request Body:**

```json
{
  "fullViewingKey": "penumbra1fvk...",
  "transactionHash": "0xabc123..."
}
```

**Response:** Returns a single transaction object with details about counterparties, assets, and metadata.

### 2. Multiple Transaction Disclosure

```
POST /disclose/transactions
```

Discloses information about multiple transactions in a single request.

**Request Body:**

```json
{
  "fullViewingKey": "penumbra1fvk...",
  "transactionHashes": ["0xabc123...", "0xdef456..."]
}
```

**Response:** Returns an array of transaction results, including both successful disclosures and any errors encountered.

### Response Types

Both endpoints return transaction data in a standardized format that includes:

- Transaction hash
- Protocol (penumbra or solana_confidential_transaction)
- Chain ID
- Counterparties (senders and receivers)
- Asset details (identifier, amount, decimals)
- Timestamp
- Transaction metadata

## Transactions

For any generated disclosure bundle, the main data type is the `Transaction` type, which includes:

- Transaction hash of the disclosed transaction
- The chain id the transaction was sent on
- Counterparties involved
  - Whether the address is a sender or receiver
  - The assets involved, including asset identifiers, amounts, and decimals
- Metadata used to describe the different action views of a transaction

For example the JSON representation of a simple token transfer disclosure

```json
{
  "transactionHash": "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf",
  "protocol": "penumbra",
  "chainId": "penumbra-testnet-phobos-x3b26d34a",
  "counterparties": [
    {
      "role": "receiver",
      "address": "penumbra147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahh09cxmz",
      "assets": [
        {
          "identifier": "wtest_usd",
          "amount": "100000000000000000000",
          "decimals": 18
        }
      ]
    },
    {
      "role": "sender",
      "address": "penumbra1alp9a75s438d33rs5nt245ue2wctfne7x4c3v7afyslmwefltgpzm7r0jgmxphrcva6h44v9pe3esstnkw5fsha54rcp7xpmaphxx76scql92mefzg366ckwcy425s3y5657ll",
      "assets": [
        {
          "identifier": "wtest_usd",
          "amount": "100000000000000000000",
          "decimals": 18
        }
      ]
    }
  ],
  "timestamp": "1745289093",
  "metadata": [
    {
      "transactionType": "Spend"
    }
  ]
}
```

# Docker Image

A dockerfile can be used to build the disclosure library CLI, and to subsequently run the API service. To compile the docker image you may run the following command

```shell
$> make build-penumbra-docker-release
```

# Docker Compose

After building the docker image, you can use the corresponding docker compose file to start the API. The `docker-compose.yml` file expects to be used with a locally running devnet. See the [penumbra guide](https://guide.penumbra.zone/dev/devnet-quickstart) for instructions on how to deploy the devnet.

# CLI

To build the CLI for the disclosure library run the following command

```shell
$> make build-penumbra-cli
# OR
$> make build-penumbra-cli-release
```

> **NOTE:** release build has significant performance increases when streaming the transactions from the Penumbra view client.

After building the CLI, you can generate disclosure bundles directly from the CLI for single transactions like so

```shell
$> ./penumbra-disclosure-cli --grpc-url $GRPC_URL disclose-transaction --full-viewing-key $FVK --transaction-hash $TX_HASH
```

# SDK Client

The SDK client provides a standalone client that can be used to disclose transactions. To avoid having to resynchronize the view server each time the client is used, the storage database is persisted on disk.

To facilitate use of the client with multiple different FVK's, the name of the database on disk is the SHA3 hash of the FVK in order to prevent leaking of the FVK.

# Example Query (Single Disclosure)

You can use the following curl command as a template for disclosing single transactions. You'll want to replace the `fullViewingKey` and `transactionHash` with values relevant to your own wallet.

```shell
$> curl -X POST \
    -H "Content-Type: application/json" \
    -d '{"fullViewingKey": "penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq", "transactionHash": "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf"}' \
    http://localhost:1337/disclose/transaction
{
    "disclosureTransactions": {
        "transactions": [
            {
                "transactionHash": "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf",
                "protocol": "penumbra",
                "chainId": "penumbra-testnet-phobos-x3b26d34a",
                "counterparties": [
                    {
                        "role": "receiver",
                        "address": "penumbra147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahh09cxmz",
                        "assets": [
                            {
                                "identifier": "wtest_usd",
                                "amount": "100000000000000000000",
                                "decimals": 18
                            }
                        ]
                    },
                    {
                        "role": "sender",
                        "address": "penumbra1alp9a75s438d33rs5nt245ue2wctfne7x4c3v7afyslmwefltgpzm7r0jgmxphrcva6h44v9pe3esstnkw5fsha54rcp7xpmaphxx76scql92mefzg366ckwcy425s3y5657ll",
                        "assets": [
                            {
                                "identifier": "wtest_usd",
                                "amount": "100000000000000000000",
                                "decimals": 18
                            }
                        ]
                    }
                ],
                "timestamp": "1745289093",
                "metadata": [
                    {
                        "transactionType": "Spend"
                    }
                ]
            }
        ]
    }
}
```

# Example Query (Multiple Disclosure)

You can use the following curl command as a template for disclosing multiple transactions. You'll want to replace the `fullViewingKey` and `transactionHashes` with values relevant to your own wallet.

```shell
$> curl -X POST \
    -H "Content-Type: application/json" \
    -d '{"fullViewingKey": "penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq", "transactionHash": ["c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf"]}' \
    http://localhost:1337/disclose/transactions
{
    "disclosureTransactions": {
        "transactions": [
            [{
                "transactionHash": "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf",
                "protocol": "penumbra",
                "chainId": "penumbra-testnet-phobos-x3b26d34a",
                "counterparties": [
                    {
                        "role": "receiver",
                        "address": "penumbra147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahh09cxmz",
                        "assets": [
                            {
                                "identifier": "wtest_usd",
                                "amount": "100000000000000000000",
                                "decimals": 18
                            }
                        ]
                    },
                    {
                        "role": "sender",
                        "address": "penumbra1alp9a75s438d33rs5nt245ue2wctfne7x4c3v7afyslmwefltgpzm7r0jgmxphrcva6h44v9pe3esstnkw5fsha54rcp7xpmaphxx76scql92mefzg366ckwcy425s3y5657ll",
                        "assets": [
                            {
                                "identifier": "wtest_usd",
                                "amount": "100000000000000000000",
                                "decimals": 18
                            }
                        ]
                    }
                ],
                "timestamp": "1745289093",
                "metadata": [
                    {
                        "transactionType": "Spend"
                    }
                ]
            }]
        ]
    }
}
```
