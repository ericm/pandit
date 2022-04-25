# ![Pandit](./pandit400.png?raw=true)

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
