FROM rust:1.65 as builder

ARG DOCKER_ARG_VERSION
ARG DOCKER_ARG_REV
ARG DOCKER_ARG_BRANCH
ARG DOCKER_ARG_BUILD_USER

ENV VERSION=$DOCKER_ARG_VERSION
ENV REV=$DOCKER_ARG_REV
ENV BRANCH=$DOCKER_ARG_BRANCH
ENV BUILD_USER=$DOCKER_ARG_BUILD_USER

COPY . /opt/data
WORKDIR /opt/data

RUN RUST_VERSION=$(rustc --version) cargo build --release

#########################################################################################

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /opt/data/target/release/vorboss /usr/bin/vorboss 
COPY --from=builder /opt/data/public /usr/bin/public 

WORKDIR /usr/bin

EXPOSE 8000

CMD ["vorboss"]
