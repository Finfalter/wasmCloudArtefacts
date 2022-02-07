# mlinference capability provider

Build with 'make'. Test with 'cargo test'.

## Test procedure

0. install __bindle__
1. start the __bindle server__, e.g. by executing `bindle/models/bindle_server_start.sh`
2. load up an invoice, e.g. by executing `bindle/models/bindle_client_push_invoice_and_parcels.sh`
3. Start nats, e.g. `nats-server --jetstream`
4. from `providers/mlinference` do `cargo test`
5. modify a few log lines in code
6. kill orphaned processes, e.g. by `pkill -f mlinference`
7. from `providers/mlinference` do `cargo test`
8. observe that changes are NOT taken into account

