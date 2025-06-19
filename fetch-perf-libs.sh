#!/usr/bin/env bash

PERF_LIBS_VERSION=v0.19.3
VERSION=$PERF_LIBS_VERSION-1

set -e
cd "$(dirname "$0")"

if [[ $VERSION != "$(cat target/perf-libs/.version 2> /dev/null)" ]]; then
  if [[ $(uname) != Linux ]]; then
    echo Note: Performance libraries are only available for Linux
    exit 0
  fi

  if [[ $(uname -m) != x86_64 ]]; then
    echo Note: Performance libraries are only available for x86_64 architecture
    exit 0
  fi

  rm -rf target/perf-libs
  mkdir -p target/perf-libs
  (
    set -x
    cd target/perf-libs

    if [[ -r ~/.cache/gorbagana-perf-$PERF_LIBS_VERSION.tgz ]]; then
      cp ~/.cache/gorbagana-perf-$PERF_LIBS_VERSION.tgz gorbagana-perf.tgz
    else
      curl -L --retry 5 --retry-delay 2 --retry-connrefused -o gorbagana-perf.tgz \
        https://github.com/gorbagana-labs/gorbagana-perf-libs/releases/download/$PERF_LIBS_VERSION/gorbagana-perf.tgz
    fi
    tar zxvf gorbagana-perf.tgz

    if [[ ! -r ~/.cache/gorbagana-perf-$PERF_LIBS_VERSION.tgz ]]; then
      # Save it for next time
      mkdir -p ~/.cache
      mv gorbagana-perf.tgz ~/.cache/gorbagana-perf-$PERF_LIBS_VERSION.tgz
    fi
    echo "$VERSION" > .version
  )

  # Setup symlinks so the perf-libs/ can be found from all binaries run out of
  # target/
  for dir in target/{debug,release}/{,deps/}; do
    mkdir -p $dir
    ln -sfT ../perf-libs ${dir}perf-libs
  done

fi

exit 0
