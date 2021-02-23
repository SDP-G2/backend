.DEFAULT_GOAL := build-run

# Push the latest built image to the docker hub
push:
	docker push kylecotton/sdp_backend:latest

build-run:
	-make build
	-make run

run:
	docker-compose up

update-schema:
	cd sdp-backend && cargo install sqlx-cli && cargo sqlx prepare

run-db:
	docker-compose up sdp_db

connect-db:
	psql -U postgres -h localhost -d sdp

build:
	docker-compose build

clean:
	-docker stop `docker ps -aq`
	-docker rm `docker ps -aq`
	-docker rmi -f `docker images -q`

migrations-run:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/up.sql

migrations-reset:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/down.sql

reset-database:
	psql -U postgres -d sdp -h localhost --single-transaction -a -f database/reset.sql

wipe-database:
	rm -rf ./database/volume/*

update-static:
	-rm -rf ./sdp-backend/static
	-git clone https://github.com/SDP-G2/frontend.git ./sdp-backend/static
	-rm -rf ./sdp-backend/static/.git

set-env:
	export PORT=8080
	export DATABASE_URL=postgres://postgres:password@localhost/sdp
