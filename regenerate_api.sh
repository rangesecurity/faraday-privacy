#! /bin/bash

openapi-generator-cli generate -i openapi.yaml -g rust -o out --additional-properties="avoidBoxedModels=true,generateAliasAsModel=true,preferUnsignedInt=true,style=deepObject" --skip-overwrite
mv out/src/models/*.rs crates/common/src/models
mv out/src/apis/*.rs crates/common/src/apis
rm -rf out
