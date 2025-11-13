use std::{array, future::Ready};

use async_modbus::embedded_io::read_holdings;
use embedded_io_adapters::tokio_1::FromTokio;
use tokio_modbus::{ExceptionCode, Request, server::Service};
use tokio_serial::SerialStream;

struct MyService {
    holdings: [u16; 16],
}

impl MyService {
    fn new() -> Self {
        Self {
            holdings: array::from_fn(|i| i as u16),
        }
    }
}

impl Service for MyService {
    type Request = tokio_modbus::Request<'static>;
    type Response = tokio_modbus::Response;
    type Exception = ExceptionCode;
    type Future = Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let ret = match req {
            Request::ReadHoldingRegisters(addr, qty) => {
                if (addr as usize) + (qty as usize) > self.holdings.len() {
                    Err(ExceptionCode::IllegalDataAddress)
                } else {
                    let data =
                        self.holdings[addr as usize..(addr as usize + qty as usize)].to_vec();
                    Ok(tokio_modbus::Response::ReadHoldingRegisters(data))
                }
            }
            _ => unimplemented!(),
        };

        std::future::ready(ret)
    }
}

#[tokio::test]
async fn test_server() {
    let (s0, s1) = SerialStream::pair().unwrap();

    tokio::spawn(tokio_modbus::server::rtu::Server::new(s0).serve_forever(MyService::new()));

    let serial = FromTokio::new(s1);

    let data = read_holdings(serial, 1, 8).await.unwrap();
    assert_eq!(data, [8, 9, 10]);
}
