# Single CU worker binary

## Environment variables

Required variables:

+ `GLOBAL_NONCE`: a hex string with a global nonce;
+ `DIFFICULTY`: a hex string with difficulty;
+ `CU_ID`: a hex string with compute unit (instance) ID;
+ `UPSTREAM`: an path or HTTP URL to `ccp_collector`'s or `lightmare` endpoint
+ `TEMP_DIR`: a path to directory where the worker can dump memory during epoch initialization

Optional variables:

+ `OVERRIDE_LOGICAL_CPU_LIST`: List of logical CPU ids to pin a worker thread to each. By default,
   autodetection is used.
+ `CCP_LOG`: tracing configuration

## Config file

Optional YAML config file is similar to `ccp` binary, but only `optimizations`, `logs` and `workers`
sections are used.  It is presumed to be relatively static.

## Command line arguments

The only argument is an optional path to a YAML config file.

## Deploy notes

Please note that `ccp_collector` removes and recreates its socket on restart.  For that reason, its parent directory should be shared with the CU worker container, not just socket file.
