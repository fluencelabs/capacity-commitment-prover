#! /usr/bin/env bash

export CCP_CONFIG="${CCP_CONFIG:-/Config.toml}"

exec /usr/local/bin/ccp_cu_worker ${CCP_CONFIG} "$@"
