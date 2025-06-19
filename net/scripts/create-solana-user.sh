#!/usr/bin/env bash
set -ex

[[ $(uname) = Linux ]] || exit 1
[[ $USER = root ]] || exit 1

if grep -q gorbagana /etc/passwd ; then
  echo "User gorbagana already exists"
else
  adduser gorbagana --gecos "" --disabled-password --quiet
  adduser gorbagana sudo
  adduser gorbagana adm
  echo "gorbagana ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
  id gorbagana

  [[ -r /gorbagana-scratch/id_ecdsa ]] || exit 1
  [[ -r /gorbagana-scratch/id_ecdsa.pub ]] || exit 1

  sudo -u gorbagana bash -c "
    echo 'PATH=\"/home/gorbagana/.cargo/bin:$PATH\"' > /home/gorbagana/.profile
    mkdir -p /home/gorbagana/.ssh/
    cd /home/gorbagana/.ssh/
    cp /gorbagana-scratch/id_ecdsa.pub authorized_keys
    umask 377
    cp /gorbagana-scratch/id_ecdsa id_ecdsa
    echo \"
      Host *
      BatchMode yes
      IdentityFile ~/.ssh/id_ecdsa
      StrictHostKeyChecking no
    \" > config
  "
fi
