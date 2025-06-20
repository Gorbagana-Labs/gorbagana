use {
    crate::{rpc_subscriptions::RpcSubscriptions, slot_status_notifier::SlotStatusNotifier},
    crossbeam_channel::RecvTimeoutError,
    gorbagana_ledger::blockstore::CompletedSlotsReceiver,
    gorbagana_rpc_client_api::response::SlotUpdate,
    gorbagana_time_utils::timestamp,
    std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread::{Builder, JoinHandle},
        time::Duration,
    },
};

pub const COMPLETE_SLOT_REPORT_SLEEP_MS: u64 = 100;

pub struct RpcCompletedSlotsService;
impl RpcCompletedSlotsService {
    pub fn spawn(
        completed_slots_receiver: CompletedSlotsReceiver,
        rpc_subscriptions: Arc<RpcSubscriptions>,
        slot_status_notifier: Option<SlotStatusNotifier>,
        exit: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        Builder::new()
            .name("solRpcComplSlot".to_string())
            .spawn(move || loop {
                // received exit signal, shutdown the service
                if exit.load(Ordering::Relaxed) {
                    break;
                }

                match completed_slots_receiver
                    .recv_timeout(Duration::from_millis(COMPLETE_SLOT_REPORT_SLEEP_MS))
                {
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        info!("RpcCompletedSlotService channel disconnected, exiting.");
                        break;
                    }
                    Ok(slots) => {
                        for slot in slots {
                            rpc_subscriptions.notify_slot_update(SlotUpdate::Completed {
                                slot,
                                timestamp: timestamp(),
                            });
                            if let Some(slot_status_notifier) = &slot_status_notifier {
                                slot_status_notifier.read().unwrap().notify_completed(slot);
                            }
                        }
                    }
                }
            })
            .unwrap()
    }
}
