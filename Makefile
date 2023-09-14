run:
	docker-compose up --build

restart:
	docker-compose restart

cover:
	rm -rf coverage
	mkdir -p coverage
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/coverage/%p-%m.profraw' cargo test
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore-not-existing -o ./coverage/lcov.info
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./coverage
	rm default_*

test:
	docker-compose -f docker-compose.tests.yaml up --build
