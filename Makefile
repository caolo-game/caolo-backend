.DEFAULT_GOAL := start

test:
	cargo test
	# TODO test webservice

worker:
	cargo run

start:
	docker-compose up --build

startworker:
	docker-compose up --scale web=0

startweb:
	docker-compose up --scale worker=0

buildworker:
	docker build -t docker.pkg.github.com/caolo-game/caolo-backend/caolo-worker:latest -f dockerfile.worker .

pushworker: buildworker
	docker push docker.pkg.github.com/caolo-game/caolo-backend/caolo-worker:latest

buildweb:
	docker build -t docker.pkg.github.com/caolo-game/caolo-backend/caolo-web:latest -f dockerfile.web .
pushweb: buildweb
	docker push docker.pkg.github.com/caolo-game/caolo-backend/caolo-web:latest

pushall: pushworker pushweb
