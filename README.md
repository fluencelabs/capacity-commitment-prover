## Build instructions

You need several dependencies, including a C++ compiler, hwloc, udev and Protobuf compiler.  For example:
```shell
apt install -y build-essential cmake libhwloc-dev libudev-dev pkg-config protobuf-compiler
```
Also, you need some git submodules:

```shell
git submodule init && git submodule update
```

## Capacity commitment prover (CCP)

CCP is a part of the Fluence Capacity Commitment protocol, which run on the capacity provider part to prove that a certain capacity unit was really allocated to the Fluence network during some epoch. CCP is heavily relies on RandomX algorithm. 

### Example

Try running from the project root:

```
tty1$ mkdir -p ../test
tty1$ CCP_LOG=debug cargo run --release -p ccp-main -- ./main/default.toml
<lot of logs>
tty2$ curl --data @main/examples/on_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"42"}
<cpus stay busy>
tty2$ sleep 5m
<cpus stay busy>
tty2$ curl --data @main/examples/on_no_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"45"}
<cpus are free>
```

## `ccp_cu_worker`

`ccp_cu_worker` is a lightweight CCP commitment binary intended to be run in a Kubernetes node.  It is intended to be a simple entity managed by Lightmare.

The `ccp_cu_worker` binary is configured in two ways: transient parameters are passed with the environment variables, and rarely-changing ones are defined in a TOML config file (simplified version of the CCP config file).

### `ccp_cu_worker` environment variables

All the parameters are required unless specified otherwise.

+ `GLOBAL_NONCE`, `DIFFICULTY`, `CU_ID` -- capacity commitment parameters, hex strings without prefix.  Every value is 64 hex chars (representing 32 octets). See the example below.
+ `UPSTREAM` -- a GRPC address of a Lightmare endpoint.  The worker uses it to submit proofs and stats.
+ `TEMP_DIR` -- a temporary directory to store about 256M (or more) of data during initialization.  The data is transient and doesn't need to be persisted.
+ `OVERRIDE_LOGICAL_CPU_LIST` -- an optional list of CPU id to use and pin to instead of autodetection.
+ `OOM_SCORE_ADJ`: -- an optional value to write to `/proc/$PID/oom_score_adj`.

### Temp data storage

During the RandomX dataset initialization, CCP and `ccp_cu_worker` dump the part of dataset to reduce the memory size.  It uses an unnamed temporary file opened in the `TMP_DIR` directory, to dump around 257M of data, and then to restore it.  This is a transient file.


### CPU ids autodetection

On Linux, `ccp_cu_worker` uses `/sys/fs/cgroup/cpuset.cpus` file to detect CPUs that it can use.  In some environments with some parameters it can contain, however, the list of all available CPUs.  This autodetection can be overridden by `OVERRIDE_LOGICAL_CPU_LIST` variable.


### Example

``` shell
TEMP_DIR=/tmp \
GLOBAL_NONCE=8450a3adfde4f9b98f5e7c03f43de169ad9697a1a4dac7452aee22a5d0037d9b \
DIFFICULTY=0001afffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
CU_ID=ab43505cb1fbef85e886ebda3ff95190edf376971921afda42145fe1f89da32e \
UPSTREAM=grpc://127.0.0.1:8070 \
   cargo run --release --bin ccp_cu_worker
```
