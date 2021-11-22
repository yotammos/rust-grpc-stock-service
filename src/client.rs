use stock_service::stock_service_client::StockServiceClient;
use stock_service::ListTransactionsRequest;

pub mod stock_service {
  tonic::include_proto!("stock_service");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = StockServiceClient::connect("http://[::1]:50051").await?;

  let request = tonic::Request::new(ListTransactionsRequest {
    name: "Tonic".into(),
  });

  let response = client.list_transactions(request).await?;

  // let request = tonic::Request::new(CreateTransactionRequest {
  //   symbol: String::from("SPY"),
  //   purchase_cost: 7000.0,
  // });
  //
  // let response = client.create_transaction(request).await?;

  println!("RESPONSE={:?}", response);

  Ok(())
}
