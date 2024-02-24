mod error;

use std::error::Error;
use crate::inx_handler::client::InxClient;
use crate::inx_handler::error::InxError;

#[derive(Clone, Debug)]
pub struct Inx {
    inx: InxClient<tonic::transport::Channel>,
}

pub mod proto {
    #![allow(missing_docs)]
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("inx");
}

/// Re-exports of [`tonic`] types.
pub mod tonic {
    pub use tonic::*;
}

pub use self::proto::inx_client as client;

impl Inx {
    pub async fn connect(address: String) -> Result<Self, InxError> {
        Ok(Self {
            inx: InxClient::connect(address).await?,
        })
    }
}