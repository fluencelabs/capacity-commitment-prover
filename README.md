## Capacity commitment prover (CCP)

CCP is a part of the Fluence Capacity Commitment protocol, which run on the capacity provider part to prove that a certain capacity unit was really allocated to the Fluence network during some epoch. CCP is heavily relies on RandomX algorithm. 

## Example

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
