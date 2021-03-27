.DEFAULT_GOAL := buildall
.PHONY: web worker

test-worker:
	${MAKE} -C worker test

start: web
	docker-compose up -d
	docker-compose logs -f --tail=100

web:
	docker build -t frenetiq/caolo-web:bleeding -f ./dockerfile.web .

worker:
	docker build -t frenetiq/caolo-worker:bleeding -f ./dockerfile.worker .

release:
	docker build -t frenetiq/caolo-release:bleeding -f dockerfile.release .

all: web worker release

push-web:web
	docker push frenetiq/caolo-web:bleeding

push-worker:worker
	docker push frenetiq/caolo-release:bleeding

push-release:release
	docker push frenetiq/caolo-worker:bleeding

push: push-web push-worker push-release

deploy-heroku-web:web
	docker tag frenetiq/caolo-web:bleeding registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/web

deploy-heroku-worker:worker
	docker tag frenetiq/caolo-release:bleeding registry.heroku.com/$(app)/release
	docker push registry.heroku.com/$(app)/worker

deploy-heroku-release:release
	docker tag frenetiq/caolo-worker:bleeding registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/release

deploy-heroku: deploy-heroku-web deploy-heroku-worker deploy-heroku-release
	heroku container:release web release worker -a=$(app)
