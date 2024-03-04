Try running from the project root:

```
1$ CCP_LOG=debug cargo run --release -p ccp-main -- ./main/default.toml
<lot of logs>
2$ curl --data @main/examples/on_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"42"}
<cpus stay busy>
2$ sleep 5m
2$ curl --data @main/examples/on_no_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"45"}
<cpus are free>
```