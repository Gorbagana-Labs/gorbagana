#!/usr/bin/env bash

set -e
cd "$(dirname "$0")"
SOLANA_ROOT="$(cd ../..; pwd)"

logDir="$PWD"/logs
rm -rf "$logDir"
mkdir "$logDir"

gorbaganaInstallDataDir=$PWD/releases
gorbaganaInstallGlobalOpts=(
  --data-dir "$gorbaganaInstallDataDir"
  --config "$gorbaganaInstallDataDir"/config.yml
  --no-modify-path
)

# Install all the gorbagana versions
bootstrapInstall() {
  declare v=$1
  if [[ ! -h $gorbaganaInstallDataDir/active_release ]]; then
    sh "$SOLANA_ROOT"/install/agave-install-init.sh "$v" "${gorbaganaInstallGlobalOpts[@]}"
  fi
  export PATH="$gorbaganaInstallDataDir/active_release/bin/:$PATH"
}

bootstrapInstall "edge"
agave-install-init --version
agave-install-init edge
gorbagana-gossip --version
gorbagana-dos --version

killall gorbagana-gossip || true
gorbagana-gossip spy --gossip-port 8001 > "$logDir"/gossip.log 2>&1 &
gorbaganaGossipPid=$!
echo "gorbagana-gossip pid: $gorbaganaGossipPid"
sleep 5
gorbagana-dos --mode gossip --data-type random --data-size 1232 &
dosPid=$!
echo "gorbagana-dos pid: $dosPid"

pass=true

SECONDS=
while ((SECONDS < 600)); do
  if ! kill -0 $gorbaganaGossipPid; then
    echo "gorbagana-gossip is no longer running after $SECONDS seconds"
    pass=false
    break
  fi
  if ! kill -0 $dosPid; then
    echo "gorbagana-dos is no longer running after $SECONDS seconds"
    pass=false
    break
  fi
  sleep 1
done

kill $gorbaganaGossipPid || true
kill $dosPid || true
wait || true

$pass && echo Pass
