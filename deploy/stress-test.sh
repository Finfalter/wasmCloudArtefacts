#!/usr/bin/env bash

for VARIABLE in {1..100}
do
    echo "heading for ${VARIABLE}th iteration .."
	curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
done