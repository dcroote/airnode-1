use web3::api::Eth;
use web3::types::{FilterBuilder, Log, H160};
use web3::{Transport, Web3};

#[derive(Clone, Debug, PartialEq)]
pub struct BlockBatch {
    pub from: u64,
    pub to: u64,
}

impl BlockBatch {
    pub fn status(&self) -> String {
        format!("{} ... {}", self.from, self.to)
    }
}

pub async fn get_batches<T: Transport>(
    eth: Eth<T>,
    genesis: u64,
    max: Option<u64>,
    batch_size: u64,
) -> Vec<BlockBatch> {
    let max_block: u64 = match max {
        Some(x) => x,
        None => eth
            .block_number()
            .await
            .expect("max block height failure")
            .as_u64(),
    };
    let mut from = genesis;
    let mut res = vec![];
    while from <= max_block {
        let to = if from + batch_size > max_block {
            max_block
        } else {
            from + batch_size - 1
        };
        res.push(BlockBatch { from, to });
        from = from + batch_size
    }
    res
}

#[derive(Debug, Clone)]
pub struct Scanner<T>
where
    T: Transport,
{
    pub web3: Web3<T>,
    pub chain_id: u64,
    pub min_block: u64,
    pub max_block: Option<u64>,
    pub batch_size: u64,
    pub batches: Vec<BlockBatch>,
}

impl<T> Scanner<T>
where
    T: Transport,
{
    pub async fn new(
        web3: &Web3<T>,
        min_block: u64,
        max_block: Option<u64>,
        batch_size: u64,
    ) -> anyhow::Result<Self> {
        let chain_id = match web3.eth().chain_id().await {
            Ok(x) => x.as_u64(),
            Err(e) => return Err(anyhow::Error::msg(format!("{}", e))),
        };
        let batches = get_batches(web3.eth(), min_block, max_block, batch_size).await;
        Ok(Self {
            web3: web3.clone(),
            chain_id,
            min_block,
            max_block,
            batch_size,
            batches,
        })
    }

    pub async fn query(
        &self,
        address: &H160,
        current_batch: usize,
    ) -> anyhow::Result<Option<Vec<Log>>> {
        if current_batch >= self.batches.len() {
            return Ok(None);
        }
        let b = self.batches[current_batch].clone();
        let filter = FilterBuilder::default()
            .from_block(b.from.into())
            .to_block(b.to.into())
            .address(vec![address.clone()])
            .build();
        let logs = self.web3.eth().logs(filter).await?;

        Ok(Some(logs))
    }
}
