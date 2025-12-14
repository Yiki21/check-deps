#!/bin/sh
set -e

# Run database migrations
if [ -x /app/migration ]; then
  /app/migration migrate up
fi

exec /app/check-deps
