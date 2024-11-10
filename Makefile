.PHONY: all
all: up

.PHONY: up
up:
	@docker-compose up --build

start_waiter:
	@docker-compose up --build waiter

.PHONY: dontcare
dontcare:
	docker-compose up

.PHONY: clean
clean: remove_artifacts

.PHONY: remove_artifacts
remove_artifacts:
	@rm -rf target logs

.PHONY: rebuild
rebuild:
	@docker build . -t base