.DEFAULT_GOAL := start

test:
	cargo test
	# TODO test webservice

start:
	docker-compose up --build

startworker:
	docker-compose up --scale web=0

startweb:
	docker-compose up --scale worker=0

buildworker:
	docker build -t frenetiq/caolo-worker:latest -f dockerfile.worker .

pushworker: buildworker
	docker push frenetiq/caolo-worker:latest

buildweb:
	docker build -t frenetiq/caolo-web:latest -f dockerfile.web .
pushweb: buildweb
	docker push frenetiq/caolo-web:latest

pushall: pushworker pushweb
