#!/usr/bin/env bash
set -e

_DIR=$(dirname ${BASH_SOURCE[0]})

show_help() {
cat <<_SHOW_HELP
  This program runs the mlinference api. Useful commands:

  Basics:
   $0 all                          - run everything
   $0 wipe                         - stop everything and erase all secrets

  Bindle:
   $0 bindle-start                 - set parameters and start the bindle server
   $0 bindle-create                - upload an invoice and corresponding parcels
   $0 bindle-stop                  - kill all bindle instances

  Host/actor controls:
   $0 inventory                    - show host inventory

Custom environment variables and paths should be set in ${_DIR}/env
_SHOW_HELP
}

## ---------------------------------------------------------------
## START CONFIGURATION
## ---------------------------------------------------------------

# define the IP address of the remote device
export IOT_DEVICE_IP=192.168.178.134
export HOST_DEVICE_IP=192.168.178.24

# define BINDLE, BINDLE_SERVER, BINDLE_URL, RUST_LOG, WASMCLOUD_HOST_HOME
source $_DIR/env

# allow extra time to process RPC
export WASMCLOUD_RPC_TIMEOUT_MS=8000
# enable verbose logging
export WASMCLOUD_STRUCTURED_LOGGING_ENABLED=1
export WASMCLOUD_STRUCTURED_LOG_LEVEL=debug
export RUST_LOG=debug

##
#   BINDLE
## 

# do NOT touch unless you know what you do
BINDLE_CONFIGURATION_SCRIPT="${_DIR}/../bindle/scripts/bindle_start.sh"
BINDLE_CREATION_SCRIPT="${_DIR}/../bindle/scripts/bindle_create.sh"
BINDLE_SHUTDOWN_SCRIPT="${_DIR}/../bindle/scripts/bindle_stop.sh"

##
#   WASMCLOUD HOST
##

# (defined in env)

##
#   CAPABILITY PROVIDERS
##

# oci registry - as used by wash
REG_SERVER=${HOST_DEVICE_IP}:5000
# registry server as seen by wasmcloud host. use "registry:5000" if host is in docker
#REG_SERVER_FROM_HOST=127.0.0.1:5000
REG_SERVER_FROM_HOST=${HOST_DEVICE_IP}:5000

HTTPSERVER_REF=wasmcloud.azurecr.io/httpserver:0.15.0
HTTPSERVER_ID=VAG3QITQQ2ODAOWB5TTQSDJ53XK3SHBEIFNK4AYJ5RKAX2UNSCAPHA5M

MLINFERENCE_REF=${REG_SERVER}/mlinference:0.1.0

# actor to link to httpsrever. 
INFERENCEAPI_ACTOR=${_DIR}/../actors/inferenceapi

# http configuration file. use https_config.json to enable TLS
HTTP_CONFIG=http_config.json

MODEL_CONFIG=actor_config.json

# command to base64 encode stdin to stdout
BASE64_ENC=base64

# where passwords are stored after being generated
SECRETS=.secrets
#PSQL_ROOT=.psql_root
#PSQL_APP=.psql_app
#CREATE_APP_SQL=.create_app.sql
CLUSTER_SEED=.cluster.nk

#ALL_SECRET_FILES="$SECRETS $PSQL_ROOT $PSQL_APP $SQL_CONFIG $CREATE_APP_SQL $CLUSTER_SEED"
ALL_SECRET_FILES="$SECRETS $CLUSTER_SEED"

## ---------------------------------------------------------------
## END CONFIGURATION
## ---------------------------------------------------------------

host_cmd() {
    $WASMCLOUD_HOST_HOME/bin/wasmcloud_host $@
}

# stop docker and wipe all data (database, nats cache, host provider/actors, etc.)
wipe_all() {

    cat >$SECRETS <<__WIPE
WASMCLOUD_CLUSTER_SEED=
WASMCLOUD_CLUSTER_SEED=
__WIPE

    docker-compose --env-file $SECRETS stop
    docker-compose --env-file $SECRETS rm -f

    rm -f $ALL_SECRET_FILES

    echo -n "going to stop wasmCloud host .."
    host_cmd stop || true

    ps -ef | grep mlinference | grep -v grep | awk '{print $2}' | xargs -r kill
    ps -ef | grep wasmcloud   | grep -v grep | awk '{print $2}' | xargs -r kill
    killall --quiet -KILL wasmcloud_httpserver_default || true
    killall --quiet -KILL wasmcloud_mlinference_default || true

    wash drain all

    # clear bindle cache
    rm -rf ~/.cache/bindle ~/Library/Caches/bindle
}

create_seed() {
    local _seed_type=$1
    wash keys gen -o json $_seed_type | jq -r '.seed'
}

create_secrets() {
    root_pass=$($MKPASS)
    app_pass=$($MKPASS)

    cluster_seed=$(create_seed Cluster)
    echo $cluster_seed >$CLUSTER_SEED

cat > $SECRETS << __SECRETS
WASMCLOUD_CLUSTER_SEED=$cluster_seed
__SECRETS

    # protect secret files
    chmod 600 $ALL_SECRET_FILES
}

start_bindle() {
    echo "\n[bindle-server startup]"

    if [ -z "$BINDLE_SERVER" ] || [ ! -x $BINDLE_SERVER ]; then
      echo "You must define BINDLE_HOME or BINDLE_SERVER"
      exit 1
    fi
    echo "BINDLE_SERVER is set to '${BINDLE_SERVER}'"

    if [ -z "$BINDLE_URL" ];  then
      echo "You must define BINDLE_URL"
      exit 1
    fi
    echo "BINDLE_URL is set to '${BINDLE_URL}'"

    eval '"$BINDLE_CONFIGURATION_SCRIPT"'
}

stop_bindle() {
    echo "\n[bindle-server shutdown]"

    eval '"$BINDLE_SHUTDOWN_SCRIPT"'
}

create_bindle() {   
    start_bindle

    echo "\n[bindle creation]"

    if [ -z "$BINDLE" ] || [ ! -x $BINDLE ]; then
      echo "You must define BINDLE_HOME or BINDLE"
      exit 1
    fi
    eval '"$BINDLE_CREATION_SCRIPT"'
}

# get the host id (requires wasmcloud to be running)
host_id() {
    wash ctl get hosts -o json | jq -r ".hosts[0].id"
}

# push capability provider
push_capability_provider() {
    echo "\npushing capability provider 'mlinference:0.1.0' to your local registry .."
    
    export WASMCLOUD_OCI_ALLOWED_INSECURE=${REG_SERVER_FROM_HOST}

    wash reg push $MLINFERENCE_REF ${_DIR}/../providers/mlinference/build/mlinference.par.gz --insecure
}

# # start docker services
# # idempotent
# start_services() {

#     # ensure we have secrets
#     if [ ! -f $SECRETS ]; then
#         create_secrets
#     fi

#     echo "starting containers with nats and registry .."

#     docker-compose --env-file $SECRETS up -d
#     # give things time to start
#     sleep 5

#     echo "starting wasmCloud host .."

#     # start wasmCloud host in background
#     export WASMCLOUD_OCI_ALLOWED_INSECURE=${REG_SERVER_FROM_HOST}
#     host_cmd start
# }

start_actors() {

    echo "starting actors .."
    _here=$PWD
    cd ${_DIR}/../actors
    for i in */; do
        if [ -f $i/Makefile ]; then
            make -C $i build push start
        fi
    done
    cd $_here
}

# start wasmcloud capability providers
# idempotent
start_providers() {
    local _host_id=$(host_id)

    echo "starting capability provider 'mlinference:0.1.0' to your local registry .."

    wash ctl start provider $HTTPSERVER_REF --link-name default --host-id $_host_id --timeout-ms 15000

    # make sure inference provider is built
    #make -C ${_DIR}/../providers/mlinference all

	wash ctl start provider $MLINFERENCE_REF --link-name default --host-id $_host_id --timeout-ms 15000
}

# base-64 encode file into a string
b64_encode_file() {
    cat "$1" | $BASE64_ENC | tr -d ' \r\n'
}

# link actors with providers
# idempotent
link_providers() {
    local _host_id=$(host_id)
    local _actor_id
    local _a

    # link inferenceapi actor to http server
    _actor_id=$(make -C $INFERENCEAPI_ACTOR --silent actor_id)
    wash ctl link put $_actor_id $HTTPSERVER_ID     \
        wasmcloud:httpserver config_b64=$(b64_encode_file $HTTP_CONFIG )

    # use locally-generated id, since mlinference provider isn't published yet
    MLINFERENCE_ID=$(wash par inspect -o json ${_DIR}/../providers/mlinference/build/mlinference.par.gz | jq -r '.service')

    # link inferenceapi actor to mlinference provider
    _actor_id=$(make -C $INFERENCEAPI_ACTOR --silent actor_id)
    wash ctl link put $_actor_id $MLINFERENCE_ID     \
        wasmcloud:mlinference config_b64=$(b64_encode_file $MODEL_CONFIG )
}

show_inventory() {
    wash ctl get inventory $(host_id)
}

# check config files
check_files() {

    for f in $HTTP_CONFIG; do
        if [ ! -f $f ]; then
            echo "missing file:$f"
            exit 1
        fi
    done

	# check syntax of json files
	jq < $HTTP_CONFIG >/dev/null
}

run_all() {

    # make sure we have all prerequisites installed
    ${_DIR}/checkup.sh

    if [ ! -f $SECRETS ]; then
        create_secrets
    fi
    check_files

    # start all the containers
    #start_services

    # start host console to view logs
    if [ "$1" = "--console" ] && [ -n "$TERMINAL" ]; then
        $TERMINAL -e ./run.sh host attach &
    fi

    # push capability provider to local registry
    push_capability_provider

    # build, push, and start all actors
    start_actors

    # start capability providers: httpserver and sqldb 
    start_providers

    # link providers with actors
    link_providers

    show_inventory
}

case $1 in 

    secrets ) create_secrets ;;
    wipe ) wipe_all ;;
    #start ) start_services ;;
    inventory ) show_inventory ;;
    bindle-start | start-bindle ) start_bindle ;;
    bindle-stop | stop-bindle ) stop_bindle ;;
    bindle-create | create-bindle ) create_bindle ;;
    start-actors ) start_actors ;;
    start-providers ) start_providers ;;
    link-providers ) link_providers ;;
    host ) shift; host_cmd $@ ;;
    run-all | all ) shift; run_all $@ ;;

    * ) show_help && exit 1 ;;

esac

