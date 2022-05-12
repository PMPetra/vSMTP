#!/bin/bash

set -e

metrics_vsmtp() {
    export PATH=$PATH:./target/release/
    user=$1

    pkill vsmtp || true
    pkill smtp-sink || true

    # FIXME:
    # The (-M $count)-nth transaction received by `smtp-sink` does not receive
    # a SMTP codes after the `<CRLF>.<CRLF>` command, so *only* the last mail
    # produce a "response error: incomplete response"

    hyperfine                   \
        --runs 5                \
        -L mail 1,10,100        \
        -L session 1,10,100     \
        --setup 'rm -rf ./tools/diff/generated/spool && vsmtp -c ./tools/diff/vsmtp.toml --no-daemon & sleep 1'         \
        -n 'vsmtp s={session}&m={mail}'  \
        'sudo smtp-sink -M {mail} -u '${user}' -d /tmp/smtp-sink-output/ 127.0.0.1:25 1 &     \
        smtp-source -s {session} -l 5120 -m {mail} -f vsmtp@smtp-source -N -t vsmtp@smtp-sink localhost:10025'     \
        --cleanup       \
            'pkill vsmtp &&                 \
            rm -rf ./diff/generated'        \
        --export-json diff-vsmtp.json
}

metrics_potfix() {
    user=$1

    pkill smtp-sink || true

    systemctl enable postfix

    hyperfine --runs 5          \
        -L mail 1,10,100        \
        -L session 1,10,100     \
        --setup 'systemctl restart postfix & sleep 3'   \
        -n 'postfix s={session}&m={mail}'               \
        'sudo smtp-sink -M {mail} -u '${user}' -d /tmp/smtp-sink-output/ 127.0.0.1:10025 1 &     \
        smtp-source -s {session} -l 5120 -m {mail} -f postfix@smtp-source -N -t postfix@smtp-sink 127.0.0.1:25'     \
        --export-json diff-postfix.json

    systemctl disable postfix
}

rm -rf /tmp/smtp-sink-output/

metrics_vsmtp $1
metrics_potfix $1

jq -s '.[0].results=([.[].results]|flatten)|.[0]' diff-vsmtp.json diff-postfix.json > diff.json
rm -f ./diff-vsmtp.json
rm -f ./diff-postfix.json

## You can then visualize the data with `https://github.com/sharkdp/hyperfine/tree/master/scripts`

# `python3 plot_whisker.py diff.json`
