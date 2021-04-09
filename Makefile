.DEFAULT_GOAL := buildall
.PHONY: api sim

test-sim:
	@${MAKE} -C sim test

test: test-sim

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
