#!/usr/bin/env bash

set -x
! tmux list-sessions || tmux kill-session
declare sudo=
if sudo true; then
  sudo="sudo -n"
fi

echo "pwd: $(pwd)"
for pid in gorbagana/*.pid; do
  pgid=$(ps opgid= "$(cat "$pid")" | tr -d '[:space:]')
  if [[ -n $pgid ]]; then
    $sudo kill -- -"$pgid"
  fi
done
if [[ -f gorbagana/netem.cfg ]]; then
  gorbagana/scripts/netem.sh delete < gorbagana/netem.cfg
  rm -f gorbagana/netem.cfg
fi
gorbagana/scripts/net-shaper.sh cleanup
for pattern in validator.sh boostrap-leader.sh gorbagana- remote- iftop validator client node; do
  echo "killing $pattern"
  pkill -f $pattern
done
