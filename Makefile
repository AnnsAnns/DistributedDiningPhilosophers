.PHONY: all
all: up

.PHONY: up
up:
	@docker-compose up --build

.PHONY: dontcare
dontcare:
	docker-compose up

.PHONY: clean
clean: remove_artifacts rebuild_cutlery rebuild_philosopher rebuild_waiter

.PHONY: remove_artifacts
remove_artifacts:
	@rm -rf target logs

.PHONY: rebuild_waiter
rebuild_waiter:
	@docker build -t waiter -f waiter.dockerfile .

.PHONY: rebuild_cutlery
rebuild_cutlery:
	@docker build -t cutlery -f cutlery.dockerfile .

.PHONY: rebuild_philosopher
rebuild_philosopher:
	@docker build -t philosopher -f philosopher.dockerfile .
