.DEFAULT_GOAL := run-local 

run-local: build
	docker-compose up

build:
	docker-compose build

clean:
	-docker stop `docker ps -aq`
	-docker rm `docker ps -aq`
	-docker rmi -f `docker images -q`


reset-database:
	rm -rf ./database/volume/*
