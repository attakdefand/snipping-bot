#!/usr/bin/env bash
set -euo pipefail
host="$1"
port="$2"
max="$3"
i=0
while ! nc -z "$host" "$port"; do
  i=$((i+1))
  if [ "$i" -ge "$max" ]; then
    echo "timed out waiting for $host:$port"
    exit 1
  fi
  sleep 1
done
echo "$host:$port is available"