.DEFAULT_GOAL := run

# --- DOCKER ---
# Push the latest built image to the docker hub
push:
	docker push kylecotton/sdp-backend:`git log -1 --format=%h`

# Run the entire backend system, if the sdp_backend image is not
#  available locally it will be fetched from the docker hub.
run:
	LAST_TAG=`sh last_tag.sh` docker-compose up

# Run the database in the background, then update the schema file
#   then build the latest image, then stop the database container
build: update-static run-db-background update-schema
	docker build -t kylecotton/sdp-backend:`git log -1 --format=%h` sdp-backend
	-docker stop `docker ps -aq`

# Update the sqlx-data.json file, for this the database must be running
update-schema:
	cd sdp-backend && cargo install sqlx-cli && cargo sqlx prepare

# Stop all running containers, remove all containers, delete all cached images
clean:
	-docker stop `docker ps -aq`
	-docker rm `docker ps -aq`
	-docker rmi -f `docker images -q`

# --- FRONTEND ---
# Update the frontend assets
update-static:
	-rm -rf ./sdp-backend/static
	-git clone https://github.com/SDP-G2/frontend.git ./sdp-backend/static
	-rm -rf ./sdp-backend/static/.git


# --- DATABASE ---
# Start only the database
run-db:
	docker-compose up sdp_db

# Start only the database, in the background
run-db-background:
	docker-compose up -d sdp_db


# Connect to the running database
connect-db:
	psql -U postgres -h localhost -d sdp

# Create the tables in the database
migrations-run:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/up.sql

# Drop all of the tables in the database
migrations-reset:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/down.sql

# Remove all of the data from the tables
reset-database:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/reset.sql

# Delete the entire volume, last resort debugging
wipe-database:
	rm -rf ./database/volume/*

# --- ENVIRNMENT VARS ---
# Set the envirnment variables for the app
set-env:
	export PORT=8080
	export DATABASE_URL=postgres://postgres:password@localhost/sdp
