#!/bin/sh

echo "Waiting for postgres..."

while ! nc -z db 5432; do
  sleep 0.1
done

echo "PostgreSQL started"

echo "Starting Stocks service in ${RUN_MODE} mode"
./service -c /etc/opt/stocks -s ${RUN_MODE} run
