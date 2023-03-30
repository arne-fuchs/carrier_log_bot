#!/bin/bash
# shellcheck disable=SC2164
cd "$(dirname "$0")"
# Sleep to wait until elite starts
sleep 60
cargo run