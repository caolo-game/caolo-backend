.DEFAULT_GOAL := buildall
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
	docker-compose logs -f --tail=100 worker

startweb:
	docker-compose up --scale worker=0 -d
	docker-compose logs -f --tail=100 web

worker:
	docker build -t frenetiq/caolo-worker:latest -f worker/dockerfile .

pushworker: worker
	docker push frenetiq/caolo-worker:latest

web:
	docker build -t frenetiq/caolo-web:latest -f web/dockerfile .

pushweb: web
	docker push frenetiq/caolo-web:latest

release:
	docker build -t frenetiq/caolo-release:latest -f web/dockerfile.release .

buildall: web worker

pushall: pushworker pushweb

deploy-heroku: buildall release
	docker tag frenetiq/caolo-web:latest registry.heroku.com/$(app)/web
	docker tag frenetiq/caolo-worker:latest registry.heroku.com/$(app)/worker
	docker tag frenetiq/caolo-release:latest registry.heroku.com/$(app)/release
	docker push registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/release
	heroku container:release web worker release

deploy: buildall pushall
	kubectl apply -f ./manifests -n=caolo
	kubectl rollout restart deployment.apps/caolo-web -n=caolo
	kubectl rollout restart deployment.apps/caolo-worker -n=caolo
