.DEFAULT_GOAL := buildall
.PHONY: api worker

test-worker:
	${MAKE} -C worker test

start:
	docker-compose up -d
	docker-compose logs -f --tail=100

api:
	docker build -t frenetiq/caolo-api:bleeding -f ./api.Dockerfile .

worker:
	docker build -t frenetiq/caolo-worker:bleeding -f ./worker.Dockerfile .

release:
	docker build -t frenetiq/caolo-release:bleeding -f release.Dockerfile .

all: api worker release

push: all
	docker push frenetiq/caolo-api:bleeding
	docker push frenetiq/caolo-release:bleeding
	docker push frenetiq/caolo-worker:bleeding

deploy-heroku: all
	docker tag frenetiq/caolo-api:bleeding registry.heroku.com/$(app)/web
	docker tag frenetiq/caolo-worker:bleeding registry.heroku.com/$(app)/worker
	docker tag frenetiq/caolo-release:bleeding registry.heroku.com/$(app)/release
	docker push registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/release
	heroku container:release web release worker -a=$(app)
