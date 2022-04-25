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

