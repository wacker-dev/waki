#!/usr/bin/env bash

# Script to re-vendor the WIT files.
#
# This script is also executed on CI to ensure that everything is up-to-date.
set -ex

# The make_vendor function takes a base path (e.g., "wasi") and a list
# of packages in the format "name@tag". It constructs the full destination
# path, downloads the tarballs from GitHub, extracts the relevant files, and
# removes any unwanted directories.
make_vendor() {
  local packages=$1
  local path="waki/wit/deps"

  rm -rf $path
  mkdir -p $path

  for package in $packages; do
    IFS='@' read -r repo tag <<< "$package"
    mkdir -p $path/$repo
    cached_extracted_dir="$cache_dir/$repo-$tag"

    if [[ ! -d $cached_extracted_dir ]]; then
      mkdir -p $cached_extracted_dir
      curl -sL https://github.com/WebAssembly/wasi-$repo/archive/$tag.tar.gz | \
        tar xzf - --strip-components=1 -C $cached_extracted_dir
      rm -rf $cached_extracted_dir/wit/deps*
    fi

    cp -r $cached_extracted_dir/wit/* $path/$repo
  done
}

cache_dir=$(mktemp -d)

make_vendor "
  cli@v0.2.0
  clocks@v0.2.0
  filesystem@v0.2.0
  io@v0.2.0
  random@v0.2.0
  sockets@v0.2.0
  http@v0.2.0
"

rm -rf $cache_dir
