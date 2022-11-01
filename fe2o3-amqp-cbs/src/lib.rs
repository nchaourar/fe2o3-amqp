//! Experimental implementation of AMQP 1.0 CBS extension protocol

use std::{future::Future, pin::Pin};

use token::CbsToken;

pub mod client;
pub mod constants;
pub mod put_token;
pub mod token;

pub trait CbsTokenProvider {
    type Error;

    fn get_token(
        &mut self,
        container_id: impl AsRef<str>,
        resource_id: impl AsRef<str>,
        claims: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<CbsToken, Self::Error>;
}

/// TODO: This will be updated when GAT is stablized
pub trait AsyncCbsTokenProvider {
    type Error;

    fn get_token_async(
        &mut self,
        container_id: impl AsRef<str>,
        resource_id: impl AsRef<str>,
        claims: impl IntoIterator<Item = impl Into<String>>,
    ) -> Pin<Box<dyn Future<Output = Result<CbsToken, Self::Error>> + '_>>;
}
