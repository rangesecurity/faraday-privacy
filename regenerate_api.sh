#! /bin/bash

openapi-generator-cli generate -i openapi.yaml -g rust -o out --additional-properties="avoidBoxedModels=true,generateAliasAsModel=true" --skip-overwrite
mv out/src/models/*.rs crates/common/src/models
rm -rf out
