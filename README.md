# rust-events-oriented-arch

[![Coverage Status](https://coveralls.io/repos/github/fabioDMFerreira/rust-events-oriented-arch/badge.svg?branch=main)](https://coveralls.io/github/fabioDMFerreira/rust-events-oriented-arch?branch=main)

Basic application to learn Rust and events oriented architecture.

## Features

1. **Actix web server**: Utilize Actix, a web framework for building high-performance API applications in Rust.

2. **Websockets integration**: Integrate websockets into Actix application, enabling real-time bidirectional communication between the client and server.

3. **Kafka integration**: Incorporate Kafka, a distributed streaming platform, to enable scalable and fault-tolerant event streaming.

4. **Postgres integration with Diesel**: Integrate Postgres, a open-source relational database, with Diesel, a powerful ORM and query builder for Rust, to efficiently manage and interact with application's data.

5. **Authentication using JWT**: Implement secure user authentication using JSON Web Tokens (JWT), allowing for stateless and secure authentication.

6. **Logging with env_logger**: Utilize env_logger, a flexible logging implementation for Rust, to efficiently log relevant information and debug.

7. **Postman Documentation**: Generate Postman documentation to provide an user-friendly API documentation.

8. **React application**: Develop a React front-end application to provide an interactive and dynamic user interface for your Actix backend.

9. **Containerization**: Setup to run all services locally in Docker containers and manifests to deploy to Kubernetes cluster.

## Directory Structure

- `api`: Contains the backend code for the project, including the API endpoints and database migrations.
- `consumer`: Contains the code for a consumer that listens for events.
- `frontend`: Contains the UI code for the project.

[Provide a brief summary of the purpose and contents of each directory.]

## Important Files and Directories

- `Makefile`: [Briefly describe the purpose of the Makefile.]
- `docker-compose.yaml`: [Briefly describe the purpose of the Docker Compose file.]
- `README`: [Briefly describe the purpose of the README file.]

[Provide any additional information or instructions related to specific files or directories.]

## Installation Requirements

- Docker

## Usage

```
$ make run
```

You can access the frontend application by opening your web browser and visiting http://localhost:3000.

The API for the web project is served on http://localhost:8000.

**Note:** Starting Kafka and Zookeeper may encounter issues and fail sometimes. In such cases, you can follow these steps to resolve the problem:

1. First, try the `make restart` command. If this doesn't solve the issue, proceed to the next step.
2. Stop the execution of `make run` and try again.
3. Lastly, reset Kafka and Zookeeper:
   3.1. Delete the data directories located at `data/kafka` and `data/zookeeper`.
   3.2. Remove the old Kafka and Zookeeper containers.
   3.3. Retry starting Kafka and Zookeeper.

## References

Special thanks to the contributors of the following repositories:

- https://github.com/wpcodevo/rust-jwt-hs256
- https://github.com/microsoft/cookiecutter-rust-actix-clean-architecture
- https://github.com/ArtRand/kafka-actix-example
- https://github.com/actix/examples
