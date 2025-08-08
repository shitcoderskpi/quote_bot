#!/bin/sh

mkdir -p /core
chmod 777 /core

exec gdb -batch \
     -ex "set pagination off" \
     -ex "run" \
     -ex "bt full" \
     --args /app/generator "$@" \
     > /core/backtrace.txt 2>&1