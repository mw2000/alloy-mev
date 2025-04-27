use alloy::{
    eips::eip2718::Encodable2718,
    network::{Ethereum, Network},
    primitives::{Bytes, B256},
    providers::{
        fillers::{FillProvider, TxFiller},
        Provider,
    },
    rpc::types::mev::{
        CancelBundleRequest, EthCallBundle, EthCallBundleResponse, EthSendBundle,
        PrivateTransactionRequest, SendBundleResponse,
    },
    transports::{http::Http, TransportErrorKind, TransportResult},
};
use async_trait::async_trait;

use crate::{BroadcastableCall, Endpoints, EndpointsBuilder, EthBundle, EthMevProviderExt};

/// A [`EthBundle`] on Ethereun network using Reqwest HTTP transport.
pub type EthereumReqwestEthBundle<'a, P> = EthBundle<'a, P, Http<reqwest::Client>, Ethereum>;

#[async_trait]
impl<F, P, N> EthMevProviderExt<reqwest::Client, N> for FillProvider<F, P, N>
where
    F: TxFiller<N>,
    P: Provider<N>,
    N: Network,
    <N as Network>::TxEnvelope: Encodable2718 + Clone,
{
    fn endpoints_builder(&self) -> EndpointsBuilder<reqwest::Client> {
        let transport = self.client().transport();
        match transport.as_any().downcast_ref::<Http<reqwest::Client>>() {
            Some(http) => EndpointsBuilder::new(http.clone()),
            None => panic!("Expected Http<reqwest::Client> transport"),
        }
    }

    async fn encode_request(&self, tx: N::TransactionRequest) -> TransportResult<Bytes> {
        let sendable = self.fill(tx).await?;

        if let Some(envelope) = sendable.as_envelope() {
            Ok(envelope.encoded_2718().into())
        } else {
            Err(TransportErrorKind::custom_str("No signer has been setup"))
        }
    }

    fn build_bundle(&self) -> EthBundle<'_, Self, Http<reqwest::Client>, N> {
        EthBundle::new(self)
    }

    async fn send_eth_bundle(
        &self,
        bundle: EthSendBundle,
        endpoints: &Endpoints,
    ) -> Vec<TransportResult<SendBundleResponse>> {
        BroadcastableCall::new(
            endpoints,
            self.client().make_request("eth_sendBundle", (bundle,)),
        )
        .await
    }

    async fn send_eth_private_transaction(
        &self,
        request: PrivateTransactionRequest,
    ) -> TransportResult<B256> {
        self.client()
            .request("eth_sendPrivateTransaction", (request,))
            .await
    }

    async fn call_eth_bundle(
        &self,
        bundle: EthCallBundle,
        endpoints: &Endpoints,
    ) -> Vec<TransportResult<EthCallBundleResponse>> {
        BroadcastableCall::new(
            endpoints,
            self.client().make_request("eth_callBundle", (bundle,)),
        )
        .await
    }

    async fn cancel_eth_bundle(&self, request: CancelBundleRequest) -> TransportResult<()> {
        self.client().request("eth_cancelBundle", (request,)).await
    }
}
