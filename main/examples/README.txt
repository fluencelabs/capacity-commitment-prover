From the project root:

1$ RUST_LOG=debug cargo run --release -p ccp-main -- --bind-address 127.0.0.1:9383 \
     --threads-per-physical-core 1 --dir-to-store-proofs a --dir-to-store-persistent-state b \
     --utility-core-id 8 --tokio-core-id 8
<lot of logs>
2$ curl --data @main/examples/on_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"42"}
<cpus stay busy>
2$ sleep 1h
2$ curl --data @main/examples/on_no_active_commitment.json -H 'content-type: application/json' http://localhost:9383
{"jsonrpc":"2.0","result":null,"id":"45"}
<cpus are free>
