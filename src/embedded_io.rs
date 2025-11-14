//! Client functions for [`embedded_io_async`]-based IO.

use embedded_io_async::{Read, ReadExactError, Write};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{
    Error, request,
    response::{self, Response},
};

/// Read multiple holding registers from a Modbus device.
pub async fn read_holdings<const N: usize, E>(
    mut serial: impl Read<Error = E> + Write<Error = E>,
    addr: u8,
    starting_register: u16,
) -> Result<[u16; N], Error<E>> {
    let req = request::ReadHoldings::new(addr, starting_register, N as u16);
    write_message(&mut serial, &req).await.map_err(Error::Io)?;

    let res: response::ReadHoldings<N> = read_message(&mut serial).await?;
    let data = res.into_data(&req)?;
    Ok(data.map(|holding| holding.get()))
}

/// Write a single holding register to a Modbus device.
pub async fn write_holding<E>(
    mut serial: impl Read<Error = E> + Write<Error = E>,
    addr: u8,
    register: u16,
    value: u16,
) -> Result<(), Error<E>> {
    let req = request::WriteHolding::new(addr, register, value);
    write_message(&mut serial, &req).await.map_err(Error::Io)?;

    let res: response::WriteHolding = read_message(&mut serial).await?;
    Ok(res.into_data(&req)?)
}

/// Write multiple holding registers to a Modbus device.
pub async fn write_holdings<const N: usize, E>(
    mut serial: impl Read<Error = E> + Write<Error = E>,
    addr: u8,
    starting_register: u16,
    data: [u16; N],
) -> Result<(), Error<E>> {
    let req = request::WriteHoldings::new(addr, starting_register, data);
    write_message(&mut serial, &req).await.map_err(Error::Io)?;

    let res: response::WriteHoldings = read_message(&mut serial).await?;
    Ok(res.into_data(&req)?)
}

/// Read multiple input registers from a Modbus device.
pub async fn read_inputs<const N: usize, E>(
    mut serial: impl Read<Error = E> + Write<Error = E>,
    addr: u8,
    starting_register: u16,
) -> Result<[u16; N], Error<E>> {
    let req = request::ReadInputs::new(addr, starting_register, N as u16);
    write_message(&mut serial, &req).await.map_err(Error::Io)?;

    let res: response::ReadInputs<N> = read_message(&mut serial).await?;
    let data = res.into_data(&req)?;
    Ok(data.map(|input| input.get()))
}

async fn write_message<T, E>(mut dst: impl Write<Error = E>, message: &T) -> Result<(), E>
where
    T: IntoBytes + Immutable,
{
    dst.write_all(message.as_bytes()).await?;
    dst.flush().await
}

async fn read_message<T, E>(mut src: impl Read<Error = E>) -> Result<T, ReadExactError<E>>
where
    T: FromBytes + IntoBytes,
{
    let mut message = T::new_zeroed();
    src.read_exact(message.as_mut_bytes()).await?;
    Ok(message)
}

impl<E> From<ReadExactError<E>> for crate::Error<E> {
    fn from(e: ReadExactError<E>) -> Self {
        match e {
            ReadExactError::Other(e) => Self::Io(e),
            ReadExactError::UnexpectedEof => Self::UnexpectedEof,
        }
    }
}
