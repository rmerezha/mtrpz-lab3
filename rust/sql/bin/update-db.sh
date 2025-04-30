#!/bin/bash

BASE_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../.. && pwd )

if [ "$BASE_DIR" == "/" ]; then
   BASE_DIR=""
fi

SCRIPTS_DIR="$BASE_DIR/sql/scripts"
LOGS_DIR="$BASE_DIR/sql/logs"
VERSION_NAME="speer-test"

while [[ $# -ge 1 ]]
do
    key="$1"

    case $key in
         --env)
            ENV_FILE="$2"
            shift
            ;;
         -h|--host)
            DB_HOST="$2"
            shift
            ;;
         -P|--port)
            DB_PORT="$2"
            shift
            ;;
         -u|--user)
            DB_USER="$2"
            shift
            ;;
         -p|--password)
            DB_PASSWORD="$2"
            shift
            ;;
         -d|--database)
            DB_NAME="$2"
            shift
            ;;
         *)
            echo "Unknown option: $key" >&2
            exit 1
            ;;
    esac
    shift
done



# Check required parameters
if [ "$DB_HOST" == "" ]; then
    echo "Host parameter is required (-h or --host)" >&2
    echo "Example: -h localhost" >&2
    exit 1
fi

if [ "$DB_NAME" == "" ]; then
    echo "Database name parameter is required (-d or --database)" >&2
    echo "Example: -d arranger" >&2
    exit 1
fi

if [ "$DB_USER" == "" ]; then
    echo "User parameter is required (-u or --user)" >&2
    echo "Example: -u arranger" >&2
    exit 1
fi

if [ "$DB_PASSWORD" == "" ]; then
    echo "Password is required (-p or --password)" >&2
    echo "Example: -p sdfa8685xxfdd" >&2
    exit 1
fi

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to start..."
until pg_isready -h "$DB_HOST" -U "$DB_USER" -p "${DB_PORT:-5432}" > /dev/null 2>&1; do
  echo "PostgreSQL is unavailable - sleeping..."
  sleep 1
done

echo "PostgreSQL is ready!"

# Set up psql arguments
PSQL="psql"
PSQLARGS="-U $DB_USER $DB_NAME"

if [ "$DB_HOST" != "" ]; then
    PSQLARGS="-h $DB_HOST $PSQLARGS"
fi

if [ "$DB_PORT" != "" ]; then
    PSQLARGS="-p $DB_PORT $PSQLARGS"
fi

export PGPASSWORD="$DB_PASSWORD"
PSQL1="$PSQL -A -t $PSQLARGS"
PSQL2="$PSQL -e -a $PSQLARGS"

# Check connection
res=$(echo "SELECT 123;" | $PSQL1)

if [ "$res" = "123" ]; then
    echo "Connection is OK"
else
    echo "Failed to connect to the database."
    exit 1
fi

# Get current version
ver=$(echo "SELECT version FROM db_version WHERE name = '$VERSION_NAME';" | ($PSQL1 2>/dev/null))
if [ "$ver" = "" ]; then
    echo "Current version: N/A"
    ver=0
else
    if [[ "$ver" =~ ^-?[0-9]+$ ]]; then
        echo "Current version: $ver"
    else
        echo "Invalid version: $ver" >&2
        exit 1
    fi
fi

((ver++))

# Apply migrations
if [ -f "$SCRIPTS_DIR/update_v$ver.sql" ]; then
    mkdir -p "$LOGS_DIR"
else
    echo "Database is up to date"
    exit 0
fi

pushd "$SCRIPTS_DIR" >/dev/null
while [ -f "update_v$ver.sql" ]
do
    echo "Updating to version $ver"
    upd="$PSQL2 -v ON_ERROR_STOP=1 -f update_v$ver.sql"
    $upd >"$LOGS_DIR/update_v$ver.log"

    if [ $? -ne 0 ] ; then
        echo "FAILURE" >&2
        echo "Check logs ($LOGS_DIR/update_v$ver.log)" >&2
        popd
        exit 1
    fi

    ((ver++))
done
popd >/dev/null

echo "DONE"
ver=$(echo "SELECT version FROM db_version WHERE name = '$VERSION_NAME';" | $PSQL1 2>/dev/null)
echo "Current version: $ver"
