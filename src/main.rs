use tonic::{transport::Server, Request, Response, Status};
use vector::vector_service_server::{VectorService, VectorServiceServer};
use vector::{
    VectorRequest, VectorResponse, InsertVectorRequest, InsertVectorResponse, 
    GetVectorByKeyRequest, InsertSampleRequest, InsertSampleResponse,
};
use scylla::{Session, SessionBuilder};
use dotenv::dotenv;
use std::env;
use futures_util::stream::TryStreamExt;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tokio::sync::Mutex;
use rand::Rng;

// Protobufで定義されたモジュール
pub mod vector {
    tonic::include_proto!("vector");
}

#[derive(Debug)]
pub struct MyVectorService {
    session: Session,
}

#[tonic::async_trait]
impl VectorService for MyVectorService {
    async fn get_vector(
        &self,
        request: Request<VectorRequest>,
    ) -> Result<Response<VectorResponse>, Status> {
        let id = request.into_inner().id;

        // CQLクエリの実行
        let query = "SELECT vector FROM vectors WHERE id = ?";
        let result = self.session.query_iter(query, (id,)).await.map_err(|e| {
            eprintln!("Database query error: {}", e);
            Status::internal("Internal server error")
        })?;

        let mut stream = result.rows_stream::<(Vec<f32>,)>().map_err(|e| {
            eprintln!("Error creating rows stream: {}", e);
            Status::internal("Internal server error")
        })?;

        if let Some(row) = stream.try_next().await.map_err(|e| {
            eprintln!("Stream error: {}", e);
            Status::internal("Internal server error")
        })? {
            let vector = row.0;
            if vector.len() != 256 {
                return Err(Status::invalid_argument("Vector dimension is not 256"));
            }

            let reply = VectorResponse {
                id,
                vector,
            };

            Ok(Response::new(reply))
        } else {
            Err(Status::not_found("Vector not found"))
        }
    }


    async fn insert_vector(
        &self,
        request: Request<InsertVectorRequest>,
    ) -> Result<Response<InsertVectorResponse>, Status> {
        let insert_req = request.into_inner();
        let id = insert_req.key;
        let key = format!("key_{}", id);
        let vector = insert_req.vector;

        if vector.len() != 256 {
            return Err(Status::invalid_argument("Vector dimension must be 256"));
        }

        // CQL INSERTクエリの実行
        let query = "INSERT INTO vectors (id, key, vector) VALUES (?, ?, ?)";
        let result = self.session.query_unpaged(query, (id, key, vector)).await.map_err(|e| {
            eprintln!("Database insert error: {}", e);
            Status::internal("Internal server error")
        })?;

        // INSERTが成功したかを確認
        if let Err(e) = result.result_not_rows() {
            eprintln!("Expected no rows in result, but got some: {}", e);
            return Err(Status::internal("Unexpected rows in result"));
        }

        let reply = InsertVectorResponse { success: true };

        Ok(Response::new(reply))
    }

    // 他のメソッドも同様に修正
    async fn get_vector_by_key(
        &self,
        request: Request<GetVectorByKeyRequest>,
    ) -> Result<Response<VectorResponse>, Status> {
        let get_req = request.into_inner();
        let key = get_req.key;

        // CQL SELECTクエリの実行
        let query = "SELECT id, vector FROM vectors WHERE key = ?";
        let result = self.session.query_iter(query, (key,)).await.map_err(|e| {
            eprintln!("Database query error: {}", e);
            Status::internal("Internal server error")
        })?;

        let mut stream = result.rows_stream::<(i32, Vec<f32>)>().map_err(|e| {
            eprintln!("Error creating rows stream: {}", e);
            Status::internal("Internal server error")
        })?;

        if let Some(row) = stream.try_next().await.map_err(|e| {
            eprintln!("Stream error: {}", e);
            Status::internal("Internal server error")
        })? {
            let (id, vector) = row;
            if vector.len() != 256 {
                return Err(Status::invalid_argument("Vector dimension is not 256"));
            }

            let reply = VectorResponse {
                id,
                vector,
            };

            Ok(Response::new(reply))
        } else {
            Err(Status::not_found("Vector not found for the given key"))
        }
    }

    async fn insert_sample(
        &self,
        request: Request<InsertSampleRequest>,
    ) -> Result<Response<InsertSampleResponse>, Status> {
        let insert_sample_req = request.into_inner();
        let id = insert_sample_req.id;
        let key = format!("auto_key_{}", id);

        // rngをlockして乱数を生成
        let rng: Mutex<StdRng> = Mutex::new(StdRng::from_entropy()).into();
        let mut rng = rng.lock().await;
        let vector: Vec<f32> = (0..256).map(|_| rng.gen_range(0.0..1.0)).collect();

        // CQL INSERTクエリの実行
        let query = "INSERT INTO vectors (id, key, vector) VALUES (?, ?, ?)";
        let result = self.session.query_unpaged(query, (id, key, vector)).await.map_err(|e| {
            eprintln!("Database insert error: {}", e);
            Status::internal("Internal server error")
        })?;

        // INSERTが成功したかを確認
        if let Err(e) = result.result_not_rows() {
            eprintln!("Expected no rows in result, but got some: {}", e);
            return Err(Status::internal("Unexpected rows in result"));
        }

        let reply = InsertSampleResponse { success: true };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // 環境変数の読み込み
    let contact_points = env::var("SCYLLA_CONTACT_POINTS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("SCYLLA_PORT").unwrap_or_else(|_| "9042".to_string()).parse()?;
    let keyspace = env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "vector_keyspace".to_string());

    // Check if the environment variables are set
    println!("SCYLLA_CONTACT_POINTS: {}", contact_points);
    println!("SCYLLA_PORT: {}", port);
    println!("SCYLLA_KEYSPACE: {}", keyspace);

    // ScyllaDBセッションの構築
    let session: Session = SessionBuilder::new()
        .known_node(format!("{}:{}", contact_points, port))
        .build()
        .await?;

    // キー空間の使用
    session.use_keyspace(&keyspace, false).await?;

    let vector_service = MyVectorService {
        session,
    };

    let addr = "0.0.0.0:50051".parse()?;
    println!("VectorService listening on {}", addr);

    Server::builder()
        .add_service(VectorServiceServer::new(vector_service))
        .serve(addr)
        .await?;

    Ok(())
}

