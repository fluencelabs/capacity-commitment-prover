#! /usr/bin/env bash

export CCP_UID=${CCP_UID:-1000}
export CCP_BASE_DIR="${CCP_BASE_DIR:-/fluence}"
export CCP_CONFIG="${CCP_CONFIG:-$CCP_BASE_DIR/Config.toml}"

useradd --uid "$CCP_UID" --gid 100 --no-create-home --shell /usr/sbin/nologin fluence >&2

mkdir -p ${CCP_BASE_DIR}
chown -R ${CCP_UID}:100 ${CCP_BASE_DIR}

exec gosu fluence ccp ${FLUENCE_CONFIG} "$@"
