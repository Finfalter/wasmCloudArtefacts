#!/bin/sh
set -e

show_help() {
cat <<_SHOW_HELP
  This program runs the mlinference api. Useful commands:

  Basics:
   $0 all                          - run everything
   $0 wipe                         - stop everything and erase all secrets

  Bindle:
   $0 start-bindle                 - set parameters and start the bindle server
   $0 create-bindle                - upload an invoice and corresponding parcels
   $0 stop-bindle                  - kill all bindle instances

  Host/actor controls:
   $0 inventory                    - show host inventory

_SHOW_HELP
}

## ---------------------------------------------------------------
## START CONFIGURATION
## ---------------------------------------------------------------

##
#   BINDLE
## 

# do NOT touch unless you know what you do
BINDLE_CONFIGURATION_SCRIPT="../bindle/models/bindle_start.sh"
BINDLE_CREATION_SCRIPT="../bindle/models/bindle_create.sh"
BINDLE_SHUTDOWN_SCRIPT="../bindle/models/bindle_stop.sh"

# set this to match the path of your bindle installation
BINDLE_HOME=~/dev/rust/bindle

##
#   WASMCLOUD HOST
##

# following a recommendation, wasmCloud host is NOT started in a container
# provide the path to wasmCloud here
WASMCLOUD_HOST_HOME=~/dev/wasmcloud/wasmCloudHost

# for 'stop-application.sh'
export WASMCLOUD_HOST_HOME=$WASMCLOUD_HOST_HOME

##
#   CAPABILITY PROVIDERS
##

HTTPSERVER_REF=wasmcloud.azurecr.io/httpserver:0.14.5
HTTPSERVER_ID=VAG3QITQQ2ODAOWB5TTQSDJ53XK3SHBEIFNK4AYJ5RKAX2UNSCAPHA5M

MLINFERENCE_REF=127.0.0.1:5000/mlinference:0.1.0
MLINFERENCE_ID=VDIRCLM2EUPU7JASBU7CWAXHBXCSYR7VAD2UZ5MZJUA47KPMQDOPTCB5

# the registry using container name
REG_SERVER=registry:5000

# actor to link to httpsrever. 
INFERENCEAPI_ACTOR=../actors/inferenceapi

# http configuration file. use https_config.json to enable TLS
HTTP_CONFIG=http_config.json

MODEL_CONFIG=actor_config.toml

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

# stop docker and wipe all data (database, nats cache, host provider/actors, etc.)
wipe_all() {

    cat >$SECRETS <<__WIPE
WASMCLOUD_CLUSTER_SEED=
WASMCLOUD_CLUSTER_SEED=
__WIPE

    docker-compose --env-file $SECRETS stop
    docker-compose --env-file $SECRETS rm -f
    wash drain all

    rm -f $ALL_SECRET_FILES

    echo -n "going to stop wasmCloud host .."

    $WASMCLOUD_HOST_HOME/bin/wasmcloud_host stop
}

create_seed() {
    local _seed_type=$1
    wash key gen -o json $_seed_type | jq -r '.seed'
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

validate_variable() {
    echo -n "Parameter '$1' is .. "

    if [ -n "${1+set}" ] && [ -d $1 ] 
    then
        echo "valid"
        local return=0
    else
        echo "NOT found"
        echo "please use '${1}' to provide a valid path to your bindle servers"
        local return=2
    fi
    echo "$return"
}

start_bindle() {
    echo "\n[bindle-server startup]"

    result=$(validate_variable "$BINDLE_HOME")

    if [ ! "$result"=0 ]; then 
        echo "'BINDLE_HOME' is invalid or not set, aborting .."
        exit 1;
    fi

    eval '"$BINDLE_CONFIGURATION_SCRIPT" http://localhost:8081/v1/ ${BINDLE_HOME}/target/debug/bindle-server'
}

stop_bindle() {
    echo "\n[bindle-server shutdown]"

    eval '"$BINDLE_SHUTDOWN_SCRIPT"'

    echo "done"
}

create_bindle() {   
    start_bindle

    echo "\n[bindle creation]"

    result=$(validate_variable "$BINDLE_HOME")

    if [ ! "$result"=0 ]; then 
        echo "'BINDLE_HOME' is invalid or not set, aborting .."
        exit 1;
    fi

    eval '"$BINDLE_CREATION_SCRIPT" http://localhost:8081/v1/ ${BINDLE_HOME}/target/debug/bindle'
}

# get the host id (requires wasmcloud to be running)
host_id() {
    wash ctl get hosts -o json | jq -r ".hosts[0].id"
}

# push capability provider
push_capability_provider() {
    echo "\npushing capability provider 'mlinference:0.1.0' to your local registry .."
    
    export WASMCLOUD_OCI_ALLOWED_INSECURE=127.0.0.1:5000

    wash reg push $MLINFERENCE_REF ../providers/mlinference/build/mlinference.par.gz --insecure
}

# start docker services
# idempotent
start_services() {

    # ensure we have secrets
    if [ ! -f $SECRETS ]; then
        create_secrets
    fi

    echo "starting containers with nats and registry .."

    docker-compose --env-file $SECRETS up -d
    # give things time to start
    sleep 5

    echo "starting wasmCloud host .."

    # start wasmCloud host
    #$WASMCLOUD_HOST_HOME/bin/wasmcloud_host foreground
    $WASMCLOUD_HOST_HOME/bin/wasmcloud_host start

}

# start wasmcloud capability providers
# idempotent
start_providers() {
    local _host_id=$(host_id)

    echo "starting capability provider 'mlinference:0.1.0' to your local registry .."

  	wash ctl start provider $HTTPSERVER_REF --link-name default --host-id $_host_id --timeout-ms 15000
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

    # link inferenceapi actor to mlinference provider
    _actor_id=$(make -C $INFERENCEAPI_ACTOR --silent actor_id)
    wash ctl link put $_actor_id $MLINFERENCE_ID     \
        wasmcloud:interfaces:mlinference config_b64=$(b64_encode_file $MODEL_CONFIG )
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
    ./checkup.sh

    if [ ! -f $SECRETS ]; then
        create_secrets
    fi
    check_files

    # start all the containers
    start_services

    # push capability provider to local registry
    push_capability_provider

    # build all actors
    make

    # push actors to registry
    make push

	# start actors
	make start REG_SERVER=127.0.0.1:5000

    # start capability providers: httpserver and sqldb 
    start_providers

    # link providers with actors
    link_providers
}

case $1 in 

    secrets ) create_secrets ;;
    wipe ) wipe_all ;;
    start ) start_services ;;
    inventory ) show_inventory ;;
    start-bindle ) start_bindle ;;
    stop-bindle ) stop_bindle ;;
    create-bindle ) create_bindle ;;
    start-providers ) start_providers ;;
    link-providers ) link_providers ;;
    run-all | all ) run_all ;;

    * ) show_help && exit 1 ;;

esac

