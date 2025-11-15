use std::{array, future::Ready, sync::Mutex};

use async_modbus::embedded_io::{read_holdings, read_inputs, write_holding, write_holdings};
use embedded_io_adapters::tokio_1::FromTokio;
use tokio_modbus::{ExceptionCode, Request, server::Service};
use tokio_serial::SerialStream;

struct MyService {
    holdings: Mutex<[u16; 16]>,
    inputs: Mutex<[u16; 16]>,
}

impl MyService {
    fn new() -> Self {
        Self {
            holdings: Mutex::new(array::from_fn(|i| i as u16)),
            inputs: Mutex::new(array::from_fn(|i| 0xa000 + i as u16)),
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
                let holdings = self.holdings.lock().unwrap();

                if (addr as usize) + (qty as usize) > holdings.len() {
                    Err(ExceptionCode::IllegalDataAddress)
                } else {
                    let data = holdings[addr as usize..(addr as usize + qty as usize)].to_vec();
                    Ok(tokio_modbus::Response::ReadHoldingRegisters(data))
                }
            }
            Request::WriteMultipleRegisters(addr, values) => {
                let mut holdings = self.holdings.lock().unwrap();

                if (addr as usize) + values.len() > holdings.len() {
                    Err(ExceptionCode::IllegalDataAddress)
                } else {
                    for (i, v) in values.iter().enumerate() {
                        holdings[addr as usize + i] = *v;
                    }
                    Ok(tokio_modbus::Response::WriteMultipleRegisters(
                        addr,
                        values.len() as u16,
                    ))
                }
            }
            Request::WriteSingleRegister(addr, value) => {
                let mut holdings = self.holdings.lock().unwrap();

                if (addr as usize) < holdings.len() {
                    holdings[addr as usize] = value;
                    Ok(tokio_modbus::Response::WriteSingleRegister(addr, value))
                } else {
                    Err(ExceptionCode::IllegalDataAddress)
                }
            }
            Request::ReadInputRegisters(addr, qty) => {
                let inputs = self.inputs.lock().unwrap();

                if (addr as usize) + (qty as usize) > inputs.len() {
                    Err(ExceptionCode::IllegalDataAddress)
                } else {
                    let data = inputs[addr as usize..(addr as usize + qty as usize)].to_vec();
                    Ok(tokio_modbus::Response::ReadInputRegisters(data))
                }
            }
            _ => unimplemented!(),
        };

        std::future::ready(ret)
    }
}

#[tokio::test]
async fn test_server() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server) = SerialStream::pair()?;

    tokio::spawn(tokio_modbus::server::rtu::Server::new(server).serve_forever(MyService::new()));

    let mut s = FromTokio::new(client);

    assert_eq!(read_holdings(&mut s, 1, 4).await?, [4, 5, 6, 7]);
    write_holding(&mut s, 1, 4, 104).await?;
    write_holdings(&mut s, 1, 6, [59]).await?;
    assert_eq!(read_holdings(&mut s, 1, 2).await?, [2, 3, 104, 5, 59, 7, 8]);

    assert_eq!(read_inputs(&mut s, 1, 0).await?, [40_960, 40_961]);

    Ok(())
}
