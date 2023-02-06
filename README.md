# Simple Paxos Implementation

## How to run it

Run the following commands in different terminals:

```bash
cargo run -- --id 0
cargo run -- --id 1
cargo run -- --id 2
```

Send a propose to any of the nodes:

```bash
curl -X POST http://0.0.0.0:8000/ -d "proposed value"
```

## References

https://lamport.azurewebsites.net/pubs/paxos-simple.pdf

https://people.cs.rutgers.edu/~pxk/417/notes/paxos.html
