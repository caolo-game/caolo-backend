.DEFAULT_GOAL := start
.PHONY: web worker

test:
	cargo check
	cargo clippy
	cargo test --benches

start: worker web
	docker-compose up -d 
	docker-compose logs -f --tail=100

startworker:
	docker-compose up --scale web=0 -d
	docker-compose logs -f --tail=100

startweb:
	docker-compose up --scale worker=0 -d
	docker-compose logs -f --tail=100

worker:
	docker build -t frenetiq/caolo-worker:latest -f dockerfile.worker .

pushworker:
	docker push frenetiq/caolo-worker:latest

web:
	docker build -t frenetiq/caolo-web:latest -f dockerfile.web .

pushweb:
	docker push frenetiq/caolo-web:latest

buildall: web worker

pushall: pushworker pushweb

deploy: buildall pushall
	git push heroku master

deploy-okteto:
	okteto build -t frenetiq/caolo-web -f dockerfile.web .
	okteto build -t frenetiq/caolo-worker -f dockerfile.worker .
	kubectl apply -f ./manifests
