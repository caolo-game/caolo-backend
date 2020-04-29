.DEFAULT_GOAL := start

test:
	cargo check
	cargo clippy
	cargo test --benches
	# TODO test webservice

start: buildworker buildweb
	docker-compose up -d 
	docker-compose logs -f --tail=100

startworker:
	docker-compose up --scale web=0 -d
	docker-compose logs -f --tail=100

startweb:
	docker-compose up --scale worker=0 -d
	docker-compose logs -f --tail=100

buildworker:
	docker build -t frenetiq/caolo-worker:latest -f dockerfile.worker .

pushworker:
	docker push frenetiq/caolo-worker:latest

buildweb:
	docker build -t frenetiq/caolo-web:latest -f dockerfile.web .

pushweb:
	docker push frenetiq/caolo-web:latest

buildall: buildweb buildworker

pushall: pushworker pushweb

deploy: buildall pushall
	git push heroku master

migrate:
	docker-compose exec web python manage.py db upgrade

protopy:
	protoc -Iprotos --python_out=webservice/build/ protos/*.proto 

bench:
	cargo bench --bench simulation_benchmarks -- --baseline master

bench-save-baseline:
	cargo bench --bench simulation_benchmarks -- --save-baseline master

deploy-okteto:
	okteto build -t frenetiq/caolo-web -f dockerfile.web .
	okteto build -t frenetiq/caolo-worker -f dockerfile.worker .
	kubectl apply -f ./manifests
