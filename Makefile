run_watch:
	cargo watch -x check -x run
test_watch:
	cargo watch -x check -x test
test_log:
	TEST_LOG=true cargo test | bunyan
docker:
	docker build -t zero2prod -f ./deployments/Dockerfile .
