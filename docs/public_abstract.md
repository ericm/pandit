## Introduction

**Pandit** transfers data on a high level between applications on different servers.
It runs an instance of itself on each server and listens for requests by applications on that server.

There are two types of applications that run on the servers: clients and 'servers'.
Clients query data and 'servers' provide said data.
Pandit acts as a middle-man for these queries.

## Services

Pandit provides 'services' to clients. A service is an abstraction on top of a 'server' that
just focuses on the data provided by the server.

Services are defined in a 'gRPC Protobuf' file that both the client and Pandit have access to, so both know the structure of a particular service.

Pandit handles the querying and conversion of data from the servers, allowing the clients to receive data in a uniform format.

## Caching

Since Pandit knows how to translate the data, it also knows when it gets the same request for data twice.
This allows it to store previously seen responses. If it gets the same request twice, it will return the data there and then, without adding extra load to the server.

**Keywords:** Distributed, Microservices, Proxy, Translation Layer

**Technologies:** Kubernetes, Docker, Rust, Redis, gRPC
