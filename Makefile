.DEFAULT_GOAL := buildall
.PHONY: web

test:
	cargo check
	cargo clippy
	cargo test --benches

start: web
	docker-compose up -d 
	docker-compose logs -f --tail=100

web:
	docker build -t frenetiq/caolo-web:latest -f web/dockerfile .

push: web
	docker push frenetiq/caolo-web:latest

release:
	docker build -t frenetiq/caolo-release:latest -f web/dockerfile.release .

deploy-heroku: web release
	docker tag frenetiq/caolo-web:latest registry.heroku.com/$(app)/web
	docker tag frenetiq/caolo-release:latest registry.heroku.com/$(app)/release
	docker push registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/release
