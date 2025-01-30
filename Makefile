.PHONY: all
all: up

.PHONY: up
up:
	@docker-compose up --build

start_waiter:
	@docker-compose up --build waiter

start_philosopher:
	@docker-compose up --build philosopher

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

podman_rebuild:
	@podman build . -t base

podman_compose_build_waiter:
	@podman-compose up --build waiter

.PHONY: compile_docs
compile_docs:
	@pdflatex -output-directory=docs docs/RDT.tex