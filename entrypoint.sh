#!/bin/sh
set -e

echo "Waiting for PostgreSQL to be ready..."

# Wait for database to be ready
max_retries=30
counter=0

until psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1 || [ $counter -eq $max_retries ]; do
  echo "PostgreSQL is unavailable - sleeping (attempt $counter/$max_retries)"
  sleep 2
  counter=$((counter + 1))
done

if [ $counter -eq $max_retries ]; then
  echo "Failed to connect to PostgreSQL after $max_retries attempts"
  exit 1
fi

echo "PostgreSQL is up and accepting connections!"
echo "Database URL: $DATABASE_URL"
echo "Starting Budget API..."
exec /app/budget
