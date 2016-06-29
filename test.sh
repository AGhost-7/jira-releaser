#!/usr/bin/env bash

FIXTURES_DIR=`readlink -f ./test-fixtures` \
	RUST_LOG=debug \
	RUST_BACKTRACE=1 \
	cargo test -- --nocapture
