//! Example Rust-based SBF program that tests sysvar use

#[allow(deprecated)]
use gorbagana_sysvar::recent_blockhashes::RecentBlockhashes;
use {
    gorbagana_account_info::AccountInfo,
    gorbagana_instruction::{AccountMeta, Instruction},
    gorbagana_instructions_sysvar as instructions,
    gorbagana_msg::msg,
    gorbagana_program_error::{ProgramError, ProgramResult},
    gorbagana_pubkey::Pubkey,
    gorbagana_sdk_ids::sysvar,
    gorbagana_sysvar::{
        clock::Clock,
        epoch_rewards::EpochRewards,
        epoch_schedule::EpochSchedule,
        rent::Rent,
        slot_hashes::{PodSlotHashes, SlotHashes},
        slot_history::SlotHistory,
        stake_history::{StakeHistory, StakeHistoryGetEntry, StakeHistorySysvar},
        Sysvar,
    },
};

// Adapted from `gorbagana_sysvar::get_sysvar` (private).
#[cfg(target_os = "gorbagana")]
fn sol_get_sysvar_handler<T>(dst: &mut [u8], offset: u64, length: u64) -> Result<(), ProgramError>
where
    T: Sysvar,
{
    let sysvar_id = &T::id() as *const _ as *const u8;
    let var_addr = dst as *mut _ as *mut u8;

    let result = unsafe {
        gorbagana_define_syscall::definitions::sol_get_sysvar(sysvar_id, var_addr, offset, length)
    };

    match result {
        gorbagana_program_entrypoint::SUCCESS => Ok(()),
        e => Err(e.into()),
    }
}

// Double-helper arrangement is easier to write to a mutable slice.
fn sol_get_sysvar<T>() -> Result<T, ProgramError>
where
    T: Sysvar,
{
    #[cfg(target_os = "gorbagana")]
    {
        let len = T::size_of();
        let mut data = vec![0; len];

        sol_get_sysvar_handler::<T>(&mut data, 0, len as u64)?;

        bincode::deserialize(&data).map_err(|_| ProgramError::InvalidArgument)
    }
    #[cfg(not(target_os = "gorbagana"))]
    Err(ProgramError::UnsupportedSysvar)
}

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // `sol_get_sysvar` can eat up some heap space for calls like
    // `PodSlotHashes`, so break up the instructions.
    //
    // * 0: Fixed-size sysvars (Clock, Rent, etc.).
    // * 1: Instruction sysvar.
    // * 2: Stake History.
    // * 3: Slot Hashes.
    match instruction_data.first() {
        Some(&0) => {
            // Clock
            {
                msg!("Clock identifier:");
                sysvar::clock::id().log();
                let clock = Clock::from_account_info(&accounts[2]).unwrap();
                assert_ne!(clock, Clock::default());
                let got_clock = Clock::get()?;
                assert_eq!(clock, got_clock);
                // Syscall `sol_get_sysvar`.
                let sgs_clock = sol_get_sysvar::<Clock>()?;
                assert_eq!(clock, sgs_clock);
            }

            // Epoch Rewards
            {
                msg!("EpochRewards identifier:");
                sysvar::epoch_rewards::id().log();
                let epoch_rewards = EpochRewards::from_account_info(&accounts[10]).unwrap();
                let got_epoch_rewards = EpochRewards::get()?;
                assert_eq!(epoch_rewards, got_epoch_rewards);
                // Syscall `sol_get_sysvar`.
                let sgs_epoch_rewards = sol_get_sysvar::<EpochRewards>()?;
                assert_eq!(epoch_rewards, sgs_epoch_rewards);
            }

            // Epoch Schedule
            {
                msg!("EpochSchedule identifier:");
                sysvar::epoch_schedule::id().log();
                let epoch_schedule = EpochSchedule::from_account_info(&accounts[3]).unwrap();
                assert_eq!(epoch_schedule, EpochSchedule::default());
                let got_epoch_schedule = EpochSchedule::get()?;
                assert_eq!(epoch_schedule, got_epoch_schedule);
                // Syscall `sol_get_sysvar`.
                let sgs_epoch_schedule = sol_get_sysvar::<EpochSchedule>()?;
                assert_eq!(epoch_schedule, sgs_epoch_schedule);
            }

            // Recent Blockhashes
            #[allow(deprecated)]
            {
                msg!("RecentBlockhashes identifier:");
                sysvar::recent_blockhashes::id().log();
                let recent_blockhashes =
                    RecentBlockhashes::from_account_info(&accounts[5]).unwrap();
                assert_ne!(recent_blockhashes, RecentBlockhashes::default());
            }

            // Rent
            {
                msg!("Rent identifier:");
                sysvar::rent::id().log();
                let rent = Rent::from_account_info(&accounts[6]).unwrap();
                let got_rent = Rent::get()?;
                assert_eq!(rent, got_rent);
                // Syscall `sol_get_sysvar`.
                let sgs_rent = sol_get_sysvar::<Rent>()?;
                assert_eq!(rent, sgs_rent);
            }

            // Slot History
            // (Not fixed-size, but also not supported)
            msg!("SlotHistory identifier:");
            sysvar::slot_history::id().log();
            assert_eq!(
                Err(ProgramError::UnsupportedSysvar),
                SlotHistory::from_account_info(&accounts[8])
            );

            Ok(())
        }
        Some(&1) => {
            // Instructions
            msg!("Instructions identifier:");
            instructions::id().log();
            assert_eq!(*accounts[4].owner, sysvar::id());
            let index = instructions::load_current_index_checked(&accounts[4])?;
            let instruction =
                instructions::load_instruction_at_checked(index as usize, &accounts[4])?;
            assert_eq!(0, index);
            assert_eq!(
                instruction,
                Instruction::new_with_bytes(
                    *program_id,
                    instruction_data,
                    vec![
                        AccountMeta::new(*accounts[0].key, true),
                        AccountMeta::new(*accounts[1].key, false),
                        AccountMeta::new_readonly(*accounts[2].key, false),
                        AccountMeta::new_readonly(*accounts[3].key, false),
                        AccountMeta::new_readonly(*accounts[4].key, false),
                        AccountMeta::new_readonly(*accounts[5].key, false),
                        AccountMeta::new_readonly(*accounts[6].key, false),
                        AccountMeta::new_readonly(*accounts[7].key, false),
                        AccountMeta::new_readonly(*accounts[8].key, false),
                        AccountMeta::new_readonly(*accounts[9].key, false),
                        AccountMeta::new_readonly(*accounts[10].key, false),
                    ],
                )
            );

            Ok(())
        }
        Some(&2) => {
            // Stake History
            {
                msg!("StakeHistory identifier:");
                sysvar::stake_history::id().log();
                let _ = StakeHistory::from_account_info(&accounts[9]).unwrap();
                // Syscall `sol_get_sysvar`.
                let stake_history_sysvar = StakeHistorySysvar(1);
                assert!(stake_history_sysvar.get_entry(0).is_some());
            }

            Ok(())
        }
        Some(&3) => {
            // Slot Hashes
            {
                msg!("SlotHashes identifier:");
                sysvar::slot_hashes::id().log();
                assert_eq!(
                    Err(ProgramError::UnsupportedSysvar),
                    SlotHashes::from_account_info(&accounts[7])
                );
                // Syscall `sol_get_sysvar`.
                let pod_slot_hashes = PodSlotHashes::fetch()?;
                assert!(pod_slot_hashes.get(/* slot */ &0)?.is_some());
            }

            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
