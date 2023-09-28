run:
	docker-compose up --build

restart:
	docker-compose restart

cover:
	rm -rf coverage
	mkdir -p coverage
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/coverage/%p-%m.profraw' cargo test --all-features --no-fail-fast --lib
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore-not-existing -o ./coverage/lcov.info
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./coverage
	rm default_*

lint:
	cargo fmt
	cargo clippy

test:
	docker-compose -f docker-compose.tests.yaml up --build


build-prod:
	docker build -t ffff/rust-users-prod -f ./docker.prod/Dockerfile.users  .
	docker build -t ffff/rust-news-prod -f ./docker.prod/Dockerfile.news  .
	docker build -t ffff/rust-news-scrapper-prod -f ./docker.prod/Dockerfile.news-scrapper  .
	docker build -t ffff/rust-fe-prod -f ./docker.prod/Dockerfile.fe  .
	docker build -t ffff/rust-users-migrations-prod -f ./docker.prod/Dockerfile.users-migrations  .
	docker build -t ffff/rust-news-migrations-prod -f ./docker.prod/Dockerfile.news-migrations  .
	docker build -t ffff/rust-consumer-prod -f ./docker.prod/Dockerfile.consumer  .

deploy-k8s:
	helm upgrade --install ingress-nginx ./k8s/ingress-nginx --namespace ingress-nginx --create-namespace
	kubectl apply -f ./k8s/postgres.yaml
	kubectl apply -f ./k8s/kafka.yaml
	kubectl apply -f ./k8s/migrations.yaml
	kubectl apply -f ./k8s/deployment.yaml

reset-k8s:
	kubectl delete all --all
