VERSION ?= 0.1.0
RUST_VERSION ?= 1.30.0
REPO ?= softprops/lambda-rust
TAG ?= "$(REPO):$(VERSION)-rust-$(RUST_VERSION)"

publish: build
	docker push $(TAG)
	docker push $(REPO):latest

build:
	docker build -t $(TAG) .
	docker tag $(TAG) $(REPO):latest