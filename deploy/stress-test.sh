#!/usr/bin/env bash

ts=$(date +%s%N)

for VARIABLE in {1..100}
do
    echo ", heading for ${VARIABLE}th iteration .."
	curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
    #curl --silent POST 0.0.0.0:8078/identity -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
done

echo -e "\nTime spent in milliseconds: '$((($(date +%s%N) - $ts)/1000000))'"