# Single CU worker binary

## Environment variables

Required variables:

+ `GLOBAL_NONCE`: a hex string with a global nonce;
+ `DIFFICULTY`: a hex string with difficulty;
+ `CU_ID`: a hex string with compute unit (instance) ID;
+ `UPSTREAM_URL`: an URL to upstream that collects proofs.

Optional variables:

+ `OVERRIDE_THREADS_COUNT`: number of threads to run (1 by default, also can be changed by config,
  but the variable has the highest priority).

## Config file

Optional YAML config file is similar to `ccp` binary, but only `optimizations`, `logs` and `workers`
sections are used.  It is presumed to be relatively static.

## Command line arguments

The only argument is an optional path to a YAML config file.
