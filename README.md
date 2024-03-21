## Capacity commitment prover (CCP)

CCP is a part of the Fluence Capacity Commitment protocol, which run on the capacity provider part to prove that a certain capacity unit was really allocated to the Fluence network during some epoch. CCP is heavily relies on RandomX algorithm. 

## Example

Try running from the project root:

```
tty1$ CCP_LOG=debug cargo run --release -p ccp-main -- ./your-config.toml
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

Please note that while crate name is `ccp-main`, the binary name is `ccp`.

## Using environment variables

You can (re)define specific config values with enviroment variable.  Use variable names using following format:

```
CCP_{SECTION}_{NAME}
```

with arbitrary case, including prefix.

For `multi-word` sections or names (e.g., `rpc-endpoint`), separate words with a hyphen, maintaining uppercase
or lowercase as necessary. Do not substitute the hyphen with an underscore. Some shells may not recognize
variables names with hyphens when used in prefix position. If you prefer not to alter the global environment,
use the `env` utility:


``` sh
env CCP_RPC-ENDPOINT_HOST=localhost \
    CCP_RPC-ENDPOINT_PORT=8099 \
  ccp ./your-config.toml
```

Environment variables override corresponding values of a config file.
