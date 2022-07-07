#!/usr/bin/env bash

echo -n "stop runtime .. "
bin/wasmcloud_host stop

echo "configure context .."
source ../configure_edge.sh

echo "remove logs .."
rm var/log/erlang.log.*

echo "remove orphans .."
ps -ef | grep mlinference | grep -v grep | awk '{print $2}' | xargs -r kill
ps -ef | grep wasmcloud   | grep -v grep | awk '{print $2}' | xargs -r kill
killall --quiet -KILL wasmcloud_httpserver_default || true
killall --quiet -KILL wasmcloud_mlinference_default || true

echo "re-start runtime .."
bin/wasmcloud_host start