top_targets     ?= all clean test lint validate

REG_SERVER ?= 127.0.0.1:5000

# traverse subdirs
.ONESHELL:
ifneq ($(subdirs),)
$(top_targets)::
	for dir in $(subdirs); do \
		$(MAKE) -C $$dir $@ REG_SERVER=$(REG_SERVER); \
	done
endif

.PHONY: all clean test lint validate
