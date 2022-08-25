subdirs = ../actors/inferenceapi ../actors/imagenetpreprocessor ../actors/imagenetpostprocessor ../actors/mnistpreprocessor ../actors/mnistpostprocessor
top_targets = all push start test clean

include ../build/makefiles/common.mk

docker-start:
	$(MAKE) start REG_SERVER=registry:5000

inventory:
	wash ctl get inventory $(shell wash ctl get hosts -o json | jq -r ".hosts[0].id")

.PHONY: all push start test clean
