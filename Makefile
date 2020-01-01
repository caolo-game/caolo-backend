.DEFAULT_GOAL := start

test:
	cargo test
	# TODO test webservice

start: buildworker buildweb
	docker-compose up

startworker:
	docker-compose up --scale web=0

startweb:
	docker-compose up --scale worker=0

buildworker:
	docker build -t frenetiq/caolo-worker:latest -f dockerfile.worker .

pushworker:
	docker push frenetiq/caolo-worker:latest

buildweb:
	docker build -t frenetiq/caolo-web:latest -f dockerfile.web .

pushweb:
	docker push frenetiq/caolo-web:latest

buildall: buildweb buildworker

pushall: pushworker pushweb
