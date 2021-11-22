use std::collections::HashMap;
use tonic::{transport::Server, Request, Response, Status};

use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client, Region};
use uuid::Uuid;

use stock_service::stock_service_server::{StockService, StockServiceServer};
use stock_service::{
  CreateTransactionRequest, CreateTransactionResponse, ListTransactionsRequest,
  ListTransactionsResponse, Transaction,
};

pub mod stock_service {
  tonic::include_proto!("stock_service");
}

#[derive(Debug, Default)]
pub struct StockServiceImpl {}

fn item_to_transaction(item: &HashMap<String, AttributeValue>) -> Transaction {
  let id = item.get("id").unwrap().as_s().unwrap().to_string();
  let count = item
    .get("count")
    .unwrap()
    .as_n()
    .unwrap()
    .to_string()
    .parse::<f64>()
    .unwrap();
  let purchase_cost = item
    .get("purchaseCost")
    .unwrap()
    .as_n()
    .unwrap()
    .to_string()
    .parse::<f64>()
    .unwrap();
  let created_at = item
    .get("createdAt")
    .unwrap()
    .as_s()
    .unwrap()
    .to_string()
    .parse::<i64>()
    .unwrap();
  let symbol = item.get("symbol").unwrap().as_s().unwrap().to_string();
  Transaction {
    id,
    count,
    purchase_cost,
    created_at,
    symbol,
  }
}

#[tonic::async_trait]
impl StockService for StockServiceImpl {
  async fn list_transactions(
    &self,
    request: Request<ListTransactionsRequest>,
  ) -> Result<Response<ListTransactionsResponse>, Status> {
    println!("Got a request: {:?}", request);

    let shared_config = aws_config::from_env()
      .region(Region::new("us-east-1"))
      .load()
      .await;
    let ddb_client = Client::new(&shared_config);
    let request = ddb_client.scan().table_name("stocks");
    let resp = request.send().await.unwrap();

    let transactions: Vec<Transaction> = resp
      .items
      .unwrap()
      .iter()
      .map(|item| item_to_transaction(item))
      .collect::<Vec<Transaction>>();

    let reply = stock_service::ListTransactionsResponse { transactions };

    Ok(Response::new(reply))
  }

  async fn create_transaction(
    &self,
    request: Request<CreateTransactionRequest>,
  ) -> Result<Response<CreateTransactionResponse>, Status> {
    println!("Got a request: {:?}", request.get_ref());

    let transaction_id = Uuid::new_v4();

    let shared_config = aws_config::from_env()
      .region(Region::new("us-east-1"))
      .load()
      .await;
    let ddb_client = Client::new(&shared_config);

    let inner_req = request.into_inner();
    let request = ddb_client
      .put_item()
      .table_name("stocks")
      .item("id", AttributeValue::S(transaction_id.to_string()))
      .item("symbol", AttributeValue::S(inner_req.symbol))
      .item(
        "purchaseCost",
        AttributeValue::N(inner_req.purchase_cost.to_string()),
      )
      .item("createdAt", AttributeValue::S(String::from("5000")))
      .item("count", AttributeValue::N(5000.0.to_string()));
    request.send().await.unwrap();

    let reply = stock_service::CreateTransactionResponse {
      id: String::from(transaction_id.to_string()),
    };

    Ok(Response::new(reply))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let address = "[::1]:50051".parse()?;
  let stock_service = StockServiceImpl::default();

  Server::builder()
    .add_service(StockServiceServer::new(stock_service))
    .serve(address)
    .await?;

  Ok(())
}
