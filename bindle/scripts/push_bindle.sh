#!/usr/bin/env bash

_DIR=$(dirname ${BASH_SOURCE[0]})
source $(dirname ${BASH_SOURCE[0]})/../../deploy/env

if [ $# -lt 2 ]
  then
    echo -n "This script expects two arguments. "
    echo -n "The first argument must be the path to an invoice definition. "
    echo "The second argument must be the path to a csv-file representing the parcels of a specific model, e.g. "
    echo "${BASH_SOURCE[0]} ../models/identical_input_output.toml ../models/identical_input_output.csv"
    exit 1
fi

echo -n "validating parameters .. "
for file in "$@"
do
    if [[ ! -f ${file} ]] ; then
        echo "file '${file}' does not exist, aborting"
        exit 1
    fi
done

INVOICE=$(basename -- "$1")
INVOICE_EXTENSION="${INVOICE##*.}"

if [ "$INVOICE_EXTENSION" != "toml" ]; then
    echo $INVOICE_EXTENSION
    echo "the first parameter must be a path to an invoice definition (.toml)"
    exit 1
fi

CSV_FILE=$(basename -- "$2")
CSV_FILE_EXTENSION="${CSV_FILE##*.}"

if [ "$CSV_FILE_EXTENSION" != "csv" ]; then
    echo $CSV_FILE_EXTENSION
    echo "the second parameter must be a path to parcel descriptions (.csv)"
    exit 1
fi
echo "ok"

echo -n "pushing the invoice .. "
echo "${BINDLE} push-invoice ${1}"
$BINDLE push-invoice $1

echo

while IFS="," read -r MODEL_IDENTIFIER PARCEL MIME_TYPE
do
    echo "parsing a parcel .. "
    echo "MIME type: $MIME_TYPE"
    echo "model:     $MODEL_IDENTIFIER"
    echo "parcel:    $PARCEL"
    echo -n "pushing the parcel .. "
    echo "${BINDLE} push-file -m ${MIME_TYPE} ${MODEL_IDENTIFIER} ${PARCEL}"
    $BINDLE push-file -m $MIME_TYPE $MODEL_IDENTIFIER ${_DIR}/$PARCEL
    echo 
    $BINDLE info $MODEL_IDENTIFIER
    echo
done < <(tail -n +2 $2)

