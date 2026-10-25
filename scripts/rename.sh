#!/bin/sh
# Re-apply the serde_json -> laxjson rename. Run after merging upstream.
# `$serde_json::private::*` are serde wire tokens; keep them untouched.
set -e
cd "$(dirname "$0")/.."
find src -name '*.rs' -print0 | xargs -0 sed -i -E \
  -e 's/\$serde_json::private::/@@TOKEN@@/g' \
  -e 's/\bserde_json\b/laxjson/g' \
  -e 's/@@TOKEN@@/$serde_json::private::/g'
sed -i -E 's/\bserde_json\b/laxjson/g' build.rs
sed -i -E 's/\bserde_json\b/laxjson/g' tests/ui/*.rs
sed -i -E 's/^name = "serde_json-fuzz"/name = "laxjson-fuzz"/; s/^serde_json = /laxjson = /' fuzz/Cargo.toml
sed -i -E 's/\bserde_json::/laxjson::/g' fuzz/fuzz_targets/*.rs
