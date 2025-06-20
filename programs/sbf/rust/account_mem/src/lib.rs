//! Test mem functions

use {
    gorbagana_account_info::AccountInfo,
    gorbagana_program_error::{ProgramError, ProgramResult},
    gorbagana_program_memory::{sol_memcmp, sol_memcpy, sol_memmove, sol_memset},
    gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = [0u8; 2048];

    let account = accounts.last().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let data_len = account.try_borrow_data()?.len().wrapping_add(10240);
    let data_ptr = account.try_borrow_mut_data()?.as_mut_ptr();
    // re-create slice with resize area
    let data = unsafe { std::slice::from_raw_parts_mut(data_ptr, data_len) };

    let mut too_early = |before: usize| -> &mut [u8] {
        let data = data.as_mut_ptr().wrapping_sub(before);

        unsafe { std::slice::from_raw_parts_mut(data, data_len) }
    };

    match instruction_data[0] {
        0 => {
            // memcmp overlaps begining
            #[allow(clippy::manual_memcpy)]
            for i in 0..500 {
                buf[i] = too_early(8)[i];
            }

            sol_memcmp(too_early(8), &buf, 500);
        }
        1 => {
            // memcmp overlaps begining
            #[allow(clippy::manual_memcpy)]
            for i in 0..12 {
                buf[i] = too_early(9)[i];
            }

            sol_memcmp(&buf, too_early(9), 12);
        }
        2 => {
            // memcmp overlaps begining
            #[allow(clippy::manual_memcpy)]
            for i in 0..3 {
                buf[i] = too_early(2)[i];
            }

            // memset overlaps begin of account area
            sol_memset(too_early(2), 3, 3);
            sol_memcpy(too_early(2), &buf, 3);
        }
        3 => {
            // memcpy src overlaps begin of account
            sol_memcpy(&mut buf, too_early(3), 10);
        }
        4 => {
            // memmov src overlaps begin of account
            unsafe { sol_memmove(buf.as_mut_ptr(), too_early(3).as_ptr(), 10) };
        }
        5 => {
            // memcpy dst overlaps begin of account
            sol_memcpy(too_early(3), &buf, 10);
        }
        6 => {
            // memmov dst overlaps begin of account
            unsafe { sol_memmove(too_early(3).as_mut_ptr(), buf.as_ptr(), 10) };
        }
        7 => {
            // memmove dst overlaps begin of account, reverse order
            unsafe { sol_memmove(too_early(0).as_mut_ptr(), too_early(3).as_ptr(), 10) };
        }
        8 => {
            // memcmp overlaps end
            sol_memcmp(&buf, &data[data_len.saturating_sub(8)..], 16);
        }
        9 => {
            // memcmp overlaps end
            sol_memcmp(&data[data_len.saturating_sub(7)..], &buf, 15);
        }
        10 => {
            // memset overlaps end of account
            sol_memset(&mut data[data_len.saturating_sub(2)..], 0, 3);
        }
        11 => {
            // memcpy src overlaps end of account
            sol_memcpy(&mut buf, &data[data_len.saturating_sub(3)..], 10);
        }
        12 => {
            // memmov src overlaps end of account
            unsafe {
                sol_memmove(
                    buf.as_mut_ptr(),
                    data[data_len.saturating_sub(3)..].as_ptr(),
                    10,
                )
            };
        }
        13 => {
            // memcpy dst overlaps end of account
            sol_memcpy(&mut data[data_len.saturating_sub(3)..], &buf, 10);
        }
        14 => {
            // memmov dst overlaps end of account
            unsafe {
                sol_memmove(
                    data[data_len.saturating_sub(3)..].as_mut_ptr(),
                    buf.as_ptr(),
                    10,
                )
            };
        }
        15 => {
            // memmove dst overlaps end of account, reverse order
            unsafe {
                sol_memmove(
                    data[data_len..].as_mut_ptr(),
                    data[data_len.saturating_sub(3)..].as_mut_ptr(),
                    10,
                )
            };
        }
        _ => {}
    }

    Ok(())
}
