.DEFAULT_GOAL := buildall
.PHONY: api sim

test-worker:
	${MAKE} -C worker test

start:
	docker-compose up -d
	docker-compose logs -f --tail=100

api:
	docker build -t frenetiq/caolo-api:bleeding -f ./api.Dockerfile .

sim:
	docker build -t frenetiq/caolo-sim:bleeding -f ./sim.Dockerfile .

release:
	docker build -t frenetiq/caolo-release:bleeding -f release.Dockerfile .

all: api sim release

push: all
	docker push frenetiq/caolo-api:bleeding
	docker push frenetiq/caolo-sim:bleeding
	docker push frenetiq/caolo-release:bleeding

deploy-heroku: all
	docker tag frenetiq/caolo-api:bleeding registry.heroku.com/$(app)/web
	docker tag frenetiq/caolo-sim:bleeding registry.heroku.com/$(app)/worker
	docker tag frenetiq/caolo-release:bleeding registry.heroku.com/$(app)/release
	docker push registry.heroku.com/$(app)/web
	docker push registry.heroku.com/$(app)/worker
	docker push registry.heroku.com/$(app)/release
	heroku container:release web release worker -a=$(app)
