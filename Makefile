.DEFAULT_GOAL := all
.PHONY: api sim rt

test-sim:
	@${MAKE} -C sim test

test-rt:
	@${MAKE} -C rt test


test: test-sim test-rt

start:
	docker-compose up -d
	docker-compose logs -f --tail=100

rt:
	docker build -t frenetiq/caolo-rt:bleeding -f ./rt.Dockerfile .

api:
	docker build -t frenetiq/caolo-api:bleeding -f ./api.Dockerfile .

sim:
	docker build -t frenetiq/caolo-sim:bleeding -f ./sim.Dockerfile .

release:
	docker build -t frenetiq/caolo-release:bleeding -f release.Dockerfile .

all: api sim release rt

push_api:api
	docker push frenetiq/caolo-api:bleeding

push_sim:sim
	docker push frenetiq/caolo-sim:bleeding

push_rt:rt
	docker push frenetiq/caolo-rt:bleeding

push_release:release
	docker push frenetiq/caolo-release:bleeding

push: push_api push_release push_rt push_sim
