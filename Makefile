.PHONY: all
all: up

.PHONY: up
up:
	@docker-compose up --build

.PHONY: dontcare
dontcare:
	docker-compose up