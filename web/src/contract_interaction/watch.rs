use model::sequential_id::SequentialId;
use near_primitives::types::AccountId;

use serde::de::DeserializeOwned;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time;

use crate::network_config::NetworkConfig;

use super::view::view;

pub fn list<T, U>(
    network_config: NetworkConfig,
    contract_id: AccountId,
    method_name: String,
    args: serde_json::Value,
    duration: time::Duration,
) -> Receiver<T>
where
    T: SequentialId<U> + DeserializeOwned + Send + 'static,
    U: Ord + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<T>(16);

    tokio::spawn(async move {
        let mut interval = time::interval(duration);
        // To ensure unique items are delivered from array, keep track of the
        // "largest" item delivered thus far.
        // Assumes that new items will be "larger" than old items.
        let mut largest_overall: Option<U> = None;
        loop {
            interval.tick().await;
            let mut largest_in_round: Option<U> = None;
            let items = view(
                &network_config,
                contract_id.clone(),
                method_name.clone(),
                &args,
            )
            .await
            .ok()
            .as_ref()
            .and_then(|view| view.as_array())
            .map(|arr| {
                arr.into_iter()
                    .map(|item| serde_json::from_value::<T>(item.clone()))
                    .filter_map(|item| match item {
                        Err(ref e) => {
                            // May be intentional (e.g. filter by parse-ability)
                            println!("Error parsing item: {}", e);
                            None
                        }
                        Ok(i) => Some(i),
                    })
                    .filter(|item| {
                        // Only take items that are "larger" than those we've seen already
                        largest_overall
                            .as_ref()
                            .map_or(true, |largest_overall| &item.seq_id() > largest_overall)
                    })
                    .collect::<Vec<T>>()
            });

            if let Some(items) = items {
                for item in items {
                    // Update largest_in_round for every item
                    if let Some(ref l) = largest_in_round {
                        if &item.seq_id() > l {
                            largest_in_round = Some(item.seq_id());
                        }
                    } else {
                        largest_in_round = Some(item.seq_id());
                    }

                    match tx.send(item).await {
                        Ok(()) => {}
                        Err(e) => println!("Error sending across channel: {}", e),
                    }
                }
            }

            // Only update largest_overall after processing each new item
            if largest_in_round.is_some() {
                largest_overall = largest_in_round;
            }
        }
    });

    rx
}
