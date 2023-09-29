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
	docker build -t ffff/rust-users-prod -f ./infra/docker.prod/Dockerfile.users  .
	docker build -t ffff/rust-news-prod -f ./infra/docker.prod/Dockerfile.news  .
	docker build -t ffff/rust-news-scrapper-prod -f ./infra/docker.prod/Dockerfile.news-scrapper  .
	docker build -t ffff/rust-fe-prod -f ./infra/docker.prod/Dockerfile.fe  .
	docker build -t ffff/rust-users-migrations-prod -f ./infra/docker.prod/Dockerfile.users-migrations  .
	docker build -t ffff/rust-news-migrations-prod -f ./infra/docker.prod/Dockerfile.news-migrations  .

deploy-k8s:
	helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
	helm upgrade --install ingress-nginx ./infra/k8s/ingress-nginx --namespace ingress-nginx --create-namespace
	helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
	helm upgrade --install monitoring ./infra/k8s/kube-prometheus-stack --values=./infra/k8s/kube-prometheus-stack/myvalues.yaml
	kubectl apply -f ./infra/k8s/postgres.yaml
	kubectl apply -f ./infra/k8s/kafka.yaml
	sleep 30
	kubectl apply -f ./infra/k8s/migrations.yaml
	kubectl apply -f ./infra/k8s/deployment.yaml

reset-k8s:
	kubectl delete all --all
