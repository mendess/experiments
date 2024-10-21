#!/bin/bash
set -e
if [[ "$1" = *-h* ]]
then
    cat <<EOF
Usage:
    Server: $0
    Client: $0 SERVER_IP
EOF
    exit
fi
if [ -z "$1" ]
then
    cat | socat - tcp-listen:9000,reuseaddr | sed -r 's/(.*)/< \1/'
else
    exec 3<>/dev/tcp/"$1"/9000

    {
        while read -r -u 3 line
        do
            echo '< ' "$line"
        done
    } &

    while read -r myline
    do
        echo "$myline" >&3
    done
fi

