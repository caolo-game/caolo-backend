.DEFAULT_GOAL := buildall
.PHONY: web worker

test-worker:
	cd worker && cargo clippy
	cd worker && cargo test --benches

start: web
	docker-compose up -d 
	docker-compose logs -f --tail=100

web:
	docker build -t frenetiq/caolo-web:bleeding -f ./dockerfile.web .

worker:
	docker build -t frenetiq/caolo-worker:bleeding -f ./dockerfile.worker .

push: web worker
	docker push frenetiq/caolo-web:bleeding
	docker push frenetiq/caolo-worker:bleeding

release:
	docker build -t frenetiq/caolo-release:bleeding -f dockerfile.release .


all: web worker release

deploy-heroku: web worker release
	docker tag frenetiq/caolo-web:bleeding registry.heroku.com/$(app)/web
	docker tag frenetiq/caolo-release:bleeding registry.heroku.com/$(app)/release
	docker tag frenetiq/caolo-worker:bleeding registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/release
	heroku container:release web release worker -a=$(app)
