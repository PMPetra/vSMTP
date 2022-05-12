#!/bin/bash

set -e

# systemctl stop vsmtp || true
# systemctl disable vsmtp || true
# systemctl enable postfix
#
# /home/lala/.cargo/bin/hyperfine \
#     -L mail 1,10 \
#     -L session 1,10 \
#     --prepare 'systemctl restart postfix' \
#     'smtp-source -s {session} -l 5120 -m {mail} -c -f to@local.com -N -t postfix@smtp-sink localhost:25' \
#     --export-json diff-postfix.json \
#     --show-output

# systemctl stop postfix || true
# systemctl disable postfix || true


if pgrep vsmtp; then pkill vsmtp; fi
if pgrep smtp-sink; then pkill smtp-sink; fi


export PATH=$PATH:/home/lala/.cargo/bin/:./target/release/

hyperfine           \
    --runs 1        \
    -L mail 1       \
    -L session 1    \
    --setup 'rm -rf ./tools/diff/generated/spool && vsmtp -c ./tools/diff/vsmtp.toml --no-daemon & sleep 1'     \
    'sudo smtp-sink -M {mail} -v -c -u lala 127.0.0.1:25 1 &     \
    smtp-source -s {session} -l 5120 -m {mail} -c -f from@smtp-source -N -t vsmtp@smtp-sink localhost:10025 &       \
    wait $(pgrep vsmtp-sink) $(pgrep smtp-source)'           \
    --cleanup       \
        'pkill vsmtp &&                  \
        rm -rf ./diff/generated'        \
    --export-json diff-vsmtp.json       \
    --show-output

## how to access function `cleanup` in --cleanup command ?

jq -s '.[0].results=([.[].results]|flatten)|.[0]' diff-postfix.json diff-vsmtp.json > diff.json
rm -f ./diff-vsmtp.json
rm -f ./diff-postfix.json

## You can then visualize the data with `https://github.com/sharkdp/hyperfine/tree/master/scripts`

# `python3 plot_whisker.py diff.json`
