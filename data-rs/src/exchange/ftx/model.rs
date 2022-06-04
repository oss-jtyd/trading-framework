use crate::{ExchangeTransformerId, Identifiable, MarketData, Validator};
use barter_integration::{SubscriptionId, socket::error::SocketError, Instrument};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::model::{Direction, Trade};

/// `Ftx` message received in response to WebSocket subscription requests.
///
/// eg/ FtxResponse::Subscribed {"type": "subscribed", "channel": "trades", "market": "BTC/USDT"}
/// eg/ FtxResponse::Error {"type": "error", "code": 400, "msg": "Missing parameter \"channel\""}
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FtxSubResponse {
    Subscribed { channel: String, market: String },
    Error { msg: String },
}

impl Validator for FtxSubResponse {
    fn validate(self) -> Result<Self, SocketError>
        where
            Self: Sized,
    {
        match &self {
            FtxSubResponse::Subscribed { .. } => Ok(self),
            FtxSubResponse::Error { msg } => Err(SocketError::Subscribe(format!(
                "received failure subscription response: {}",
                msg
            ))),
        }
    }
}

/// `Ftx` Message variants that can be received over [`WebSocket`].
#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(tag = "channel", rename_all = "lowercase")]
pub enum FtxMessage {
    Trades {
        market: SubscriptionId,
        #[serde(rename = "data")]
        trades: Vec<FtxTrade>,
    },
}

impl Identifiable for FtxMessage {
    fn id(&self) -> SubscriptionId {
        match self {
            FtxMessage::Trades { market: subscription_id, .. } => subscription_id.clone(),
        }
    }
}

/// `Ftx` trade message.
#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
pub struct FtxTrade {
    pub id: u64,
    pub price: f64,
    pub size: f64,
    #[serde(rename = "side")]
    pub direction: Direction,
    pub time: DateTime<Utc>,
}

impl From<(ExchangeTransformerId, Instrument, FtxTrade)> for MarketData {
    fn from((exchange, instrument, trade): (ExchangeTransformerId, Instrument, FtxTrade)) -> Self {
        Self::Trade(Trade {
            id: trade.id.to_string(),
            exchange: exchange.exchange().to_string(),
            instrument,
            received_timestamp: Utc::now(),
            exchange_timestamp: trade.time,
            price: trade.price,
            quantity: trade.size,
            direction: trade.direction
        })
    }
}