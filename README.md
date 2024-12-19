# hajimeteno-tonic

Ref: [tonic/examples/helloworld-tutorial.md at master · hyperium/tonic](https://github.com/hyperium/tonic/blob/master/examples/helloworld-tutorial.md)

# k8s SetUp

Create Image

```shell
docker build -t hajimeteno-tonic:latest .
```

Create Namespace
```shell
kubectl create namespace hajimeteno-tonic-ns
```

Set namespace
```shell
kubectl config set-context --current --namespace=hajimeteno-tonic-ns
```


Create Deployment
```shell
kubectl apply -f k8s/scylladb-deployment.yaml
kubectl apply -f k8s/scylladb-service.yaml
kubectl apply -f k8s/grpc-deployment.yaml
kubectl apply -f k8s/grpc-service-service.yaml
```

DB SetUp

```shell
kubectl get pods -l app=scylladb
```

```shell
kubectl exec -it $(kubectl get pods -l app=scylladb -o jsonpath='{.items[0].metadata.name}') cqlsh
```

```shell
CREATE KEYSPACE IF NOT EXISTS vector_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE IF NOT EXISTS vector_keyspace.vectors (
    id int PRIMARY KEY,
    key text,
    vector list<float>
);

```

Check Grpc from localhost

```shell
grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 100
}' localhost:30051 vector.VectorService/InsertSample
```

```shell
grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 100
}' localhost:30051 vector.VectorService/GetVector
```

(if not found)

```
❯ grpcurl -plaintext -import-path src/proto -proto hello.proto -d '{
  "id": 10001
}' localhost:30051 vector.VectorService/GetVector
ERROR:
  Code: NotFound
  Message: Vector not found
```
