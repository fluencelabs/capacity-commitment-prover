# Single CU worker binary

## Environment variables

Required variables:

+ `GLOBAL_NONCE`: a hex string with a global nonce;
+ `DIFFICULTY`: a hex string with difficulty;
+ `CU_ID`: a hex string with compute unit (instance) ID;
+ `UPSTREAM_URL`: an path or HTTP URL to `ccp_collector`'s Unix domain socket
+ `TEMP_DIR`: a path to directory where the worker can dump memory during epoch initialization

Optional variables:

+ `OVERRIDE_THREADS_COUNT`: number of threads to run (1 by default, also can be changed by config,
  but the variable has the highest priority).
+ `CCP_LOG`: tracing configuration

## Config file

Optional YAML config file is similar to `ccp` binary, but only `optimizations`, `logs` and `workers`
sections are used.  It is presumed to be relatively static.

## Command line arguments

The only argument is an optional path to a YAML config file.

## Deploy notes

Please note that `ccp_collector` removes and recreates its socket on restart.  For that reason, its parent directory should be shared with the CU worker container, not just socket file.
