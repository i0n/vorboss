# vorboss

- **Deployment available at:** https://vorboss.i0n.io
- **CI:** https://app.circleci.com/pipelines/github/i0n/vorboss
- **Docker image:** https://hub.docker.com/repository/docker/i0nw/vorboss

To run locally you will need rust and/or docker.

Set the `AIRTABLE_API_KEY` env var.

    AIRTABLE_API_KEY=secret cargo run
or

    docker run --name vorboss --rm --network vorboss -e AIRTABLE_API_KEY=secret -p 8000:8000 i0nw/vorboss:latest

To run the functional tests locally you will also need k6:

    brew install k6
then

    make test-functional


