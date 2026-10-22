---
title: Use the artifacts
sidebar_label: Use the artifacts
description: Fetch a2a.proto and the OpenAPI, AsyncAPI and JSON Schema documents, verify them, and take one schema out of the bundle.
---

The documents come in two forms: as they are on `main`, from this site, and as released, from a
GitHub Release. [Downloads](../reference/downloads.md) lists both.

## From this site

```sh
base=https://beyond10x.github.io/a2a/downloads
for f in SHA256SUMS a2a.proto a2a-upstream.openapi.yaml a2a-http.openapi.yaml \
         a2a.openapi.yaml a2a.asyncapi.yaml a2a.schema.json; do
  curl -fsSLO "$base/$f"
done
sha256sum --check SHA256SUMS
```

`sha256sum --check` prints `OK` for each file and exits 0 when every file matches.

## From a release

The assets of a release carry its version in their names, and the release's own `SHA256SUMS`
covers them. Set `v` to the release you want; Downloads names the latest.

```sh
v=0.3.0
base=https://github.com/beyond10x/a2a/releases/download/$v
for f in SHA256SUMS a2a.proto a2a-$v-upstream.openapi.yaml a2a-$v.openapi.yaml a2a-$v-http.openapi.yaml \
         a2a-$v.asyncapi.yaml a2a-$v-json-schema.tar.gz a2a-$v-specification.tar.gz; do
  curl -fsSLO "$base/$f"
done
sha256sum --check SHA256SUMS
```

## One schema out of the bundle

`a2a.schema.json` is one JSON Schema 2020-12 document. Each file of `generated/schema/schema/` is
a resource under `$defs`, keyed `<directory>.<name>`, with its own `$id` relative to the bundle.
Every `$ref` in a resource points into that resource's own `$defs`, so a schema taken out of the
bundle needs nothing else.

List the keys, then take one schema out:

```sh
jq -r '."$defs" | keys[]' a2a.schema.json
jq '."$defs"["entities.a2a.tasks.Task"]' a2a.schema.json > task.schema.json
```
