# Check if a custom user has been set, otherwise default to 'postgres'
#Set-Variable DB_USER=${POSTGRES_USER:=postgres}
$DB_USER = if ($env:POSTGRES_USER) { $env:POSTGRES_USER } else { "postgres" };
# Check if a custom password has been set, otherwise default to 'password'
#Set-Variable DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
$DB_PASSWORD = if ($env:POSTGRES_PASSWORD) { $env:POSTGRES_PASSWORD } else { "password" };
# Check if a custom database name has been set, otherwise default to 'newsletter'
#Set-Variable DB_NAME="${POSTGRES_DB:=newsletter}"
$DB_NAME = if ($env:DB_NAME) { $env:DB_NAME } else { "newsletter" };
# Check if a custom port has been set, otherwise default to '5432'
#Set-Variable DB_PORT="${POSTGRES_PORT:=5432}"
$DB_PORT = if ($env:POSTGRES_PORT) { $env:POSTGRES_PORT } else { "5432" };
# Launch postgres using Docker

docker run -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=$DB_PASSWORD -e POSTGRES_DB=$DB_NAME -p ${DB_PORT}:5432 -d postgres postgres -N 1000

$env:DATABASE_URL = "postgres://${DB_USER}:${DB_PASSWORD}@127.0.0.1:${DB_PORT}/${DB_NAME}"
Write-Output $env:DATABASE_URL

Start-Sleep -Seconds 30

sqlx database create
sqlx migrate run
Write-Output "ran migrations"



