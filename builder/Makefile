VERSION ?= '0.0.0'
RUST_VERSION ?= '1.26.1'
TAG ?= 'softprops/lambda-rust:$(VERSION)-rust-$(RUST_VERSION)'

publish: build
	docker push $(TAG)

build:
	docker build -t $(TAG) .