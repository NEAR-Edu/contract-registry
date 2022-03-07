FROM rust:1.59

WORKDIR /usr/src/app

COPY . .

WORKDIR /usr/src/app/service
RUN cargo install --path .

WORKDIR /usr/src/app
CMD ["contract-registry-service"]
