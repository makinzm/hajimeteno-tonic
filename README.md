# hajimeteno-tonic

Ref: [tonic/examples/helloworld-tutorial.md at master · hyperium/tonic](https://github.com/hyperium/tonic/blob/master/examples/helloworld-tutorial.md)

# Failure

```shell
❯ grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 140
}' localhost:50051 vector.VectorService/GetVector
Error invoking method "vector.VectorService/GetVector": target server does not expose service "vector.VectorService"
```

# SetUp


## SetUp DB

[scylladb/scylla - Docker Image | Docker Hub](https://hub.docker.com/r/scylladb/scylla/)

Launch scylladb

```shell
docker pull scylladb/scylla:latest
docker run --name scylladb -d -p 9042:9042 scylladb/scylla:latest
```

Create a keyspace and a table

```shell
docker exec -it scylladb cqlsh
```

```shell
CREATE KEYSPACE IF NOT EXISTS vector_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE IF NOT EXISTS vector_keyspace.vectors (
    id int PRIMARY KEY,
    key text,
    vector list<float>
);
```

Check table

```shell
docker exec -it scylladb cqlsh
```
```shell
cqlsh> use vector_keyspace;
cqlsh:vector_keyspace> describe tables;

vectors

```

## SetUp Rust

```shell
devbox shell
cargo build
cargo run
```

## Check App

```shell
grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 100
}' localhost:50051 vector.VectorService/InsertSample
```
Expectation
```shell
{
  "success": true
}
```

```shell
grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 100
}' localhost:50051 vector.VectorService/GetVector
```

