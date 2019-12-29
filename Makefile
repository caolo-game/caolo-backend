.DEFAULT_GOAL := worker

worker:
	cargo run

startworker:
	docker-compose up --scale web=0

startservice:
	docker-compose up --scale worker=0

pushworker:
	docker build -t docker.pkg.github.com/caolo-game/caolo-backend/caolo-worker:latest -f dockerfile.worker .
	docker push docker.pkg.github.com/caolo-game/caolo-backend/caolo-worker:latest

pushweb:
	docker build -t docker.pkg.github.com/caolo-game/caolo-backend/caolo-web:latest -f dockerfile.web .
	docker push docker.pkg.github.com/caolo-game/caolo-backend/caolo-web:latest

pushall: pushworker pushweb
