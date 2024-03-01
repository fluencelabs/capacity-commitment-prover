#! /usr/bin/env bash

export FLUENCE_UID=${FLUENCE_UID:-1000}
useradd --uid "$FLUENCE_UID" --gid 100 --no-create-home --shell /usr/sbin/nologin fluence >&2

exec gosu fluence ccp "$@"
