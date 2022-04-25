# ![Pandit](./pandit400.png?raw=true)

Pandit is a distributed proxy that converts applications into [gRPC Services](https://grpc.io/docs/what-is-grpc/core-concepts/).

Its architecture allows it to translate requests and responses from gRPC to any data format the application may require.

This is is achieved by defining the translation inside the gRPC Protobuf specification. For example:
```proto
message ExampleRequest {
  int32 id = 1 [ (pandit.key) = true ]; // defines the key used to index the request in cache.
  string user = 2;
}

message ExampleResponse {
  option (pandit.path) = ".obj"; 
  // A path can be added to messgaes or fields in return messages. 
  // When the payload is parsed into an intermediary representation, 
  // this can be used to define a sub-path to parse the object/field from.
  // Path follows JQ-like syntax.
  int32 id = 1;
  string user = 2;
}

service ExampleService {
  option (pandit.name) = "my_service";
  option (pandit.format.http_service) = {
    hostname : "localhost"
    version : VERSION_1_1
  };
  // HTTP specific options, defining that this service proxies 
  // a HTTP backend.

  rpc GetExample(ExampleRequest) returns (ExampleResponse) {
    option (pandit.format.http) = {
      get : "/example" 
    };
    // HTTP-specific options for each method.
    option (pandit.handler) = JSON; 
    // This handler will mean this method will convert to a JSON payload.
    option (pandit.cache) = {
      cache_time : 3000
    };
    // Caching options used to define caching strategy for this method.
  }
}
```
## Writers
A writer is what sends the request to the application. Currently there are two available:
- [HTTP](./src/proto/format/http.proto)
- [Postgres](./src/proto/format/postgres.proto) (your millage may vary).
- More can be implemented (just implement the [Writer](https://github.com/ericm/pandit/blob/1e486ae3f78981b42e9770e6d5d1aefea626efaf/src/services/mod.rs#L735) trait)

They are responsible for interfacing with the application and encapsulating the payload in the relevant headers.
The Handler will generate the request payload and parse the response payload.

## Handlers
The handlers are responsible for serialising/deserialising payloads.
The current handlers available are: 
- [JSON](./src/handlers/json.rs)
- [SQL](./src/handlers/sql.rs)
- More can be implemented (just implement the [Handler](https://github.com/ericm/pandit/blob/1e486ae3f78981b42e9770e6d5d1aefea626efaf/src/services/mod.rs#L87) trait)

## Features
### Caching
Each daemon caches responses from previously seen requests, using the `pandit.key` as the index.
When a service is on another server, the query will be delegated. 
There will then be potential cache hits on both servers, as separate cache tables are maintained.
![image](https://user-images.githubusercontent.com/29894839/165186910-9893e7d4-c5d8-4a47-945d-5f35a904e356.png)

After a successful response, all interested pandit instances will get an updated copy of the cache:
![image](https://user-images.githubusercontent.com/29894839/165186785-bbb1b35d-c504-4832-9eaf-844ec67d398a.png)


### Docker deployment mode.
Pandit can be deployed as a daemon in a Docker environment.
When a service is added, Pandit queries the Docker API for a container
matching the provided container ID. If found, Pandit will do the following:
- It will create a Docker network specifically to facilitate communication
between Pandit, and the application.
- It will add both Pandit’s container, and the application’s container, to
the network. Both of them will be assigned an IP address.

Pandit will now be able to proxy docker container behind gRPC.

### Kubernetes deployment mode.
Pandit provides the following options for kubernetes deployment:
```proto
  oneof container {
    string k8s_pod = 5; 
    // A Pandit service can be defined as a single Pod on a Single Node.
    
    string k8s_service = 6; 
    // A Pandit service can be defined as a Kubernetes Service.
    // All Pods derived fron its Label Selector will be proxied by Pandit.
    // Each Node the Pods live on will host the Pandit Service.
    
    string k8s_replica_set = 7;
    string k8s_stateful_set = 8;
    // These operate in a similar manner to the service,
    // indexed via label selectors.
    
    // Pandit load balances between nodes automatically.
  }
}
```

Example of a Kubernetes Pod as a Pandit Service:
![image](https://user-images.githubusercontent.com/29894839/165187879-5439aaab-280d-4c63-a309-056b283eece1.png)

All Kubernetes deployments can be found in [`./k8s`](./k8s).

## Requirements
- Linux
- [Rust nightly](https://www.oreilly.com/library/view/rust-programming-by/9781788390637/e07dc768-de29-482e-804b-0274b4bef418.xhtml)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Redis server

## Setting up development environment
### Basic Mode

1. Pull the dependencies & build the daemon, CLI and examples:
    ```
    $ cargo build --workspace
    ```

1. Run the redis server:
    ```
    $ redis-server
    ```

1. In another terminal, run the daemon in with the directory of this repository as the current working directory:
    ```
    $ ./target/debug/panditd
    ```
#### Running the example 
1. Run the example REST API:
    ```
    $ cd ./src/proto/examples && ./example_rest.py
    ```

1. Add the service to the daemon with the CLI:
    ```
    $ cd cli/example && ./test_example.sh 
    ```

1. Run the example client: 
    ```
    $ ./target/debug/test_example1 
    ```

See `./cli/examples` for more examples.

### Kubernetes Mode
#### Requirements
- Minikube
- Docker (for minikube backend)
- Kubectl

#### Setup

1. Setup local Kubernetes cluster:
    ```
    $ minikube start --mount-string=${PWD}:/pandit --mount --nodes=3
    ```

1. Build the CLI to run on ubuntu:
    ```
    $ docker build -f build.Dockerfile -t pandit-build .
    $ docker run --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp/cli pandit-build cargo build --target-dir=target-ubuntu
    ```

1. Deploy the Kuberentes resources:
    ```
    $ cd k8s && kubectl apply -f redis-config.yml -f redis.yml -f debug.yml -f example.yml
    ```

1. (Optional) List the deployed pods to see which nodes they're on:
    ```
    $ kubectl get pods --all-namespaces -o wide 
    ```

#### Running examples

1. SSH into the minikube host:
    ```
    $ minikube ssh 
    ```

1. Add a service:
    ```
    $ cd /pandit/cli/example && ../target-ubuntu/debug/pandit --proto-path ../../src/proto/examples add . 
    ```

1. Test queries to the service from multiple nodes:
    ```
    $ kubectl exec -it debug-kdm87 -- /mnt/pandit/target/debug/test_example1
    1
    $ kubectl exec -it debug-fcghx -- /mnt/pandit/target/debug/test_example1
    1
    ```
