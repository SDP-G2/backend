.DEFAULT_GOAL := build-run

build-run:
	-make build
	-make run

run:
	docker-compose up

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

wipe-database:
	rm -rf ./database/volume/*

set-env:
	export PORT=8080
	export DATABASE_URL=postgres://postgres:password@localhost/sdp
