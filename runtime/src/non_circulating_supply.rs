use {
    crate::bank::Bank,
    log::*,
    gorbagana_account::ReadableAccount,
    gorbagana_accounts_db::accounts_index::{AccountIndex, IndexKey, ScanConfig, ScanResult},
    gorbagana_pubkey::Pubkey,
    gorbagana_stake_interface::{self as stake, state::StakeStateV2},
    gorbagana_stake_program::stake_state,
    std::collections::HashSet,
};

pub struct NonCirculatingSupply {
    pub lamports: u64,
    pub accounts: Vec<Pubkey>,
}

pub fn calculate_non_circulating_supply(bank: &Bank) -> ScanResult<NonCirculatingSupply> {
    debug!("Updating Bank supply, epoch: {}", bank.epoch());
    let mut non_circulating_accounts_set: HashSet<Pubkey> = HashSet::new();

    for key in non_circulating_accounts() {
        non_circulating_accounts_set.insert(key);
    }
    let withdraw_authority_list = withdraw_authority();

    let clock = bank.clock();
    let config = &ScanConfig::default();
    let stake_accounts = if bank
        .rc
        .accounts
        .accounts_db
        .account_indexes
        .contains(&AccountIndex::ProgramId)
    {
        bank.get_filtered_indexed_accounts(
            &IndexKey::ProgramId(stake::program::id()),
            // The program-id account index checks for Account owner on inclusion. However, due to
            // the current AccountsDb implementation, an account may remain in storage as a
            // zero-lamport Account::Default() after being wiped and reinitialized in later
            // updates. We include the redundant filter here to avoid returning these accounts.
            |account| account.owner() == &stake::program::id(),
            config,
            None,
        )?
    } else {
        bank.get_program_accounts(&stake::program::id(), config)?
    };

    for (pubkey, account) in stake_accounts.iter() {
        let stake_account = stake_state::from(account).unwrap_or_default();
        match stake_account {
            StakeStateV2::Initialized(meta) => {
                if meta.lockup.is_in_force(&clock, None)
                    || withdraw_authority_list.contains(&meta.authorized.withdrawer)
                {
                    non_circulating_accounts_set.insert(*pubkey);
                }
            }
            StakeStateV2::Stake(meta, _stake, _stake_flags) => {
                if meta.lockup.is_in_force(&clock, None)
                    || withdraw_authority_list.contains(&meta.authorized.withdrawer)
                {
                    non_circulating_accounts_set.insert(*pubkey);
                }
            }
            _ => {}
        }
    }

    let lamports = non_circulating_accounts_set
        .iter()
        .map(|pubkey| bank.get_balance(pubkey))
        .sum();

    Ok(NonCirculatingSupply {
        lamports,
        accounts: non_circulating_accounts_set.into_iter().collect(),
    })
}

// Mainnet-beta accounts that should be considered non-circulating
pub fn non_circulating_accounts() -> Vec<Pubkey> {
    [
        gorbagana_pubkey::pubkey!("9huDUZfxoJ7wGMTffUE7vh1xePqef7gyrLJu9NApncqA"),
        gorbagana_pubkey::pubkey!("GK2zqSsXLA2rwVZk347RYhh6jJpRsCA69FjLW93ZGi3B"),
        gorbagana_pubkey::pubkey!("CWeRmXme7LmbaUWTZWFLt6FMnpzLCHaQLuR2TdgFn4Lq"),
        gorbagana_pubkey::pubkey!("HCV5dGFJXRrJ3jhDYA4DCeb9TEDTwGGYXtT3wHksu2Zr"),
        gorbagana_pubkey::pubkey!("14FUT96s9swbmH7ZjpDvfEDywnAYy9zaNhv4xvezySGu"),
        gorbagana_pubkey::pubkey!("HbZ5FfmKWNHC7uwk6TF1hVi6TCs7dtYfdjEcuPGgzFAg"),
        gorbagana_pubkey::pubkey!("C7C8odR8oashR5Feyrq2tJKaXL18id1dSj2zbkDGL2C2"),
        gorbagana_pubkey::pubkey!("Eyr9P5XsjK2NUKNCnfu39eqpGoiLFgVAv1LSQgMZCwiQ"),
        gorbagana_pubkey::pubkey!("DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ"),
        gorbagana_pubkey::pubkey!("CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S"),
        gorbagana_pubkey::pubkey!("7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2"),
        gorbagana_pubkey::pubkey!("GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ"),
        gorbagana_pubkey::pubkey!("Mc5XB47H3DKJHym5RLa9mPzWv5snERsF3KNv5AauXK8"),
        gorbagana_pubkey::pubkey!("7cvkjYAkUYs4W8XcXsca7cBrEGFeSUjeZmKoNBvEwyri"),
        gorbagana_pubkey::pubkey!("AG3m2bAibcY8raMt4oXEGqRHwX4FWKPPJVjZxn1LySDX"),
        gorbagana_pubkey::pubkey!("5XdtyEDREHJXXW1CTtCsVjJRjBapAwK78ZquzvnNVRrV"),
        gorbagana_pubkey::pubkey!("6yKHERk8rsbmJxvMpPuwPs1ct3hRiP7xaJF2tvnGU6nK"),
        gorbagana_pubkey::pubkey!("CHmdL15akDcJgBkY6BP3hzs98Dqr6wbdDC5p8odvtSbq"),
        gorbagana_pubkey::pubkey!("FR84wZQy3Y3j2gWz6pgETUiUoJtreMEuWfbg6573UCj9"),
        gorbagana_pubkey::pubkey!("5q54XjQ7vDx4y6KphPeE97LUNiYGtP55spjvXAWPGBuf"),
        gorbagana_pubkey::pubkey!("3o6xgkJ9sTmDeQWyfj3sxwon18fXJB9PV5LDc8sfgR4a"),
        gorbagana_pubkey::pubkey!("GumSE5HsMV5HCwBTv2D2D81yy9x17aDkvobkqAfTRgmo"),
        gorbagana_pubkey::pubkey!("AzVV9ZZDxTgW4wWfJmsG6ytaHpQGSe1yz76Nyy84VbQF"),
        gorbagana_pubkey::pubkey!("8CUUMKYNGxdgYio5CLHRHyzMEhhVRMcqefgE6dLqnVRK"),
        gorbagana_pubkey::pubkey!("CQDYc4ET2mbFhVpgj41gXahL6Exn5ZoPcGAzSHuYxwmE"),
        gorbagana_pubkey::pubkey!("5PLJZLJiRR9vf7d1JCCg7UuWjtyN9nkab9uok6TqSyuP"),
        gorbagana_pubkey::pubkey!("7xJ9CLtEAcEShw9kW2gSoZkRWL566Dg12cvgzANJwbTr"),
        gorbagana_pubkey::pubkey!("BuCEvc9ze8UoAQwwsQLy8d447C8sA4zeVtVpc6m5wQeS"),
        gorbagana_pubkey::pubkey!("8ndGYFjav6NDXvzYcxs449Aub3AxYv4vYpk89zRDwgj7"),
        gorbagana_pubkey::pubkey!("8W58E8JVJjH1jCy5CeHJQgvwFXTyAVyesuXRZGbcSUGG"),
        gorbagana_pubkey::pubkey!("GNiz4Mq886bTNDT3pijGsu2gbw6it7sqrwncro45USeB"),
        gorbagana_pubkey::pubkey!("GhsotwFMH6XUrRLJCxcx62h7748N2Uq8mf87hUGkmPhg"),
        gorbagana_pubkey::pubkey!("Fgyh8EeYGZtbW8sS33YmNQnzx54WXPrJ5KWNPkCfWPot"),
        gorbagana_pubkey::pubkey!("8UVjvYyoqP6sqcctTso3xpCdCfgTMiv3VRh7vraC2eJk"),
        gorbagana_pubkey::pubkey!("BhvLngiqqKeZ8rpxch2uGjeCiC88zzewoWPRuoxpp1aS"),
        gorbagana_pubkey::pubkey!("63DtkW7zuARcd185EmHAkfF44bDcC2SiTSEj2spLP3iA"),
        gorbagana_pubkey::pubkey!("GvpCiTgq9dmEeojCDBivoLoZqc4AkbUDACpqPMwYLWKh"),
        gorbagana_pubkey::pubkey!("7Y8smnoUrYKGGuDq2uaFKVxJYhojgg7DVixHyAtGTYEV"),
        gorbagana_pubkey::pubkey!("DUS1KxwUhUyDKB4A81E8vdnTe3hSahd92Abtn9CXsEcj"),
        gorbagana_pubkey::pubkey!("F9MWFw8cnYVwsRq8Am1PGfFL3cQUZV37mbGoxZftzLjN"),
        gorbagana_pubkey::pubkey!("8vqrX3H2BYLaXVintse3gorPEM4TgTwTFZNN1Fm9TdYs"),
        gorbagana_pubkey::pubkey!("CUageMFi49kzoDqtdU8NvQ4Bq3sbtJygjKDAXJ45nmAi"),
        gorbagana_pubkey::pubkey!("5smrYwb1Hr2T8XMnvsqccTgXxuqQs14iuE8RbHFYf2Cf"),
        gorbagana_pubkey::pubkey!("xQadXQiUTCCFhfHjvQx1hyJK6KVWr1w2fD6DT3cdwj7"),
        gorbagana_pubkey::pubkey!("8DE8fqPfv1fp9DHyGyDFFaMjpopMgDeXspzoi9jpBJjC"),
        gorbagana_pubkey::pubkey!("3itU5ME8L6FDqtMiRoUiT1F7PwbkTtHBbW51YWD5jtjm"),
        gorbagana_pubkey::pubkey!("AsrYX4FeLXnZcrjcZmrASY2Eq1jvEeQfwxtNTxS5zojA"),
        gorbagana_pubkey::pubkey!("8rT45mqpuDBR1vcnDc9kwP9DrZAXDR4ZeuKWw3u1gTGa"),
        gorbagana_pubkey::pubkey!("nGME7HgBT6tAJN1f6YuCCngpqT5cvSTndZUVLjQ4jwA"),
        gorbagana_pubkey::pubkey!("CzAHrrrHKx9Lxf6wdCMrsZkLvk74c7J2vGv8VYPUmY6v"),
        gorbagana_pubkey::pubkey!("AzHQ8Bia1grVVbcGyci7wzueSWkgvu7YZVZ4B9rkL5P6"),
        gorbagana_pubkey::pubkey!("FiWYY85b58zEEcPtxe3PuqzWPjqBJXqdwgZeqSBmT9Cn"),
        gorbagana_pubkey::pubkey!("GpxpMVhrBBBEYbEJxdR62w3daWz444V7m6dxYDZKH77D"),
        gorbagana_pubkey::pubkey!("3bTGcGB9F98XxnrBNftmmm48JGfPgi5sYxDEKiCjQYk3"),
        gorbagana_pubkey::pubkey!("8pNBEppa1VcFAsx4Hzq9CpdXUXZjUXbvQwLX2K7QsCwb"),
        gorbagana_pubkey::pubkey!("HKJgYGTTYYR2ZkfJKHbn58w676fKueQXmvbtpyvrSM3N"),
        gorbagana_pubkey::pubkey!("3jnknRabs7G2V9dKhxd2KP85pNWXKXiedYnYxtySnQMs"),
        gorbagana_pubkey::pubkey!("4sxwau4mdqZ8zEJsfryXq4QFYnMJSCp3HWuZQod8WU5k"),
        gorbagana_pubkey::pubkey!("Fg12tB1tz8w6zJSQ4ZAGotWoCztdMJF9hqK8R11pakog"),
        gorbagana_pubkey::pubkey!("GEWSkfWgHkpiLbeKaAnwvqnECGdRNf49at5nFccVey7c"),
        gorbagana_pubkey::pubkey!("CND6ZjRTzaCFVdX7pSSWgjTfHZuhxqFDoUBqWBJguNoA"),
        gorbagana_pubkey::pubkey!("2WWb1gRzuXDd5viZLQF7pNRR6Y7UiyeaPpaL35X6j3ve"),
        gorbagana_pubkey::pubkey!("BUnRE27mYXN9p8H1Ay24GXhJC88q2CuwLoNU2v2CrW4W"),
        gorbagana_pubkey::pubkey!("CsUqV42gVQLJwQsKyjWHqGkfHarxn9hcY4YeSjgaaeTd"),
        gorbagana_pubkey::pubkey!("5khMKAcvmsFaAhoKkdg3u5abvKsmjUQNmhTNP624WB1F"),
        gorbagana_pubkey::pubkey!("GpYnVDgB7dzvwSgsjQFeHznjG6Kt1DLBFYrKxjGU1LuD"),
        gorbagana_pubkey::pubkey!("DQQGPtj7pphPHCLzzBuEyDDQByUcKGrsJdsH7SP3hAug"),
        gorbagana_pubkey::pubkey!("FwfaykN7ACnsEUDHANzGHqTGQZMcGnUSsahAHUqbdPrz"),
        gorbagana_pubkey::pubkey!("JCwT5Ygmq3VeBEbDjL8s8E82Ra2rP9bq45QfZE7Xyaq7"),
        gorbagana_pubkey::pubkey!("H3Ni7vG1CsmJZdTvxF7RkAf9UM5qk4RsohJsmPvtZNnu"),
        gorbagana_pubkey::pubkey!("CVgyXrbEd1ctEuvq11QdpnCQVnPit8NLdhyqXQHLprM2"),
        gorbagana_pubkey::pubkey!("EAJJD6nDqtXcZ4DnQb19F9XEz8y8bRDHxbWbahatZNbL"),
        gorbagana_pubkey::pubkey!("6o5v1HC7WhBnLfRHp8mQTtCP2khdXXjhuyGyYEoy2Suy"),
        gorbagana_pubkey::pubkey!("3ZrsTmNM6AkMcqFfv3ryfhQ2jMfqP64RQbqVyAaxqhrQ"),
        gorbagana_pubkey::pubkey!("6zw7em7uQdmMpuS9fGz8Nq9TLHa5YQhEKKwPjo5PwDK4"),
        gorbagana_pubkey::pubkey!("CuatS6njAcfkFHnvai7zXCs7syA9bykXWsDCJEWfhjHG"),
        gorbagana_pubkey::pubkey!("Hz9nydgN1k15wnwffKX7CSmZp4VFTnTwLXAEdomFGNXy"),
        gorbagana_pubkey::pubkey!("Ep5Y58PaSyALPrdFxDVAdfKtVdP55vApvsWjb3jSmXsG"),
        gorbagana_pubkey::pubkey!("EziVYi3Sv5kJWxmU77PnbrT8jmkVuqwdiFLLzZpLVEn7"),
        gorbagana_pubkey::pubkey!("H1rt8KvXkNhQExTRfkY8r9wjZbZ8yCih6J4wQ5Fz9HGP"),
        gorbagana_pubkey::pubkey!("6nN69B4uZuESZYxr9nrLDjmKRtjDZQXrehwkfQTKw62U"),
        gorbagana_pubkey::pubkey!("Hm9JW7of5i9dnrboS8pCUCSeoQUPh7JsP1rkbJnW7An4"),
        gorbagana_pubkey::pubkey!("5D5NxsNVTgXHyVziwV7mDFwVDS6voaBsyyGxUbhQrhNW"),
        gorbagana_pubkey::pubkey!("EMAY24PrS6rWfvpqffFCsTsFJypeeYYmtUc26wdh3Wup"),
        gorbagana_pubkey::pubkey!("Br3aeVGapRb2xTq17RU2pYZCoJpWA7bq6TKBCcYtMSmt"),
        gorbagana_pubkey::pubkey!("BUjkdqUuH5Lz9XzcMcR4DdEMnFG6r8QzUMBm16Rfau96"),
        gorbagana_pubkey::pubkey!("Es13uD2p64UVPFpEWfDtd6SERdoNR2XVgqBQBZcZSLqW"),
        gorbagana_pubkey::pubkey!("AVYpwVou2BhdLivAwLxKPALZQsY7aZNkNmGbP2fZw7RU"),
        gorbagana_pubkey::pubkey!("DrKzW5koKSZp4mg4BdHLwr72MMXscd2kTiWgckCvvPXz"),
        gorbagana_pubkey::pubkey!("9hknftBZAQL4f48tWfk3bUEV5YSLcYYtDRqNmpNnhCWG"),
        gorbagana_pubkey::pubkey!("GLUmCeJpXB8veNcchPwibkRYwCwvQbKodex5mEjrgToi"),
        gorbagana_pubkey::pubkey!("9S2M3UYPpnPZTBtbcUvehYmiWFK3kBhwfzV2iWuwvaVy"),
        gorbagana_pubkey::pubkey!("HUAkU5psJXZuw54Lrg1ksbXzHv2fzczQ9sNbmisVMeJU"),
        gorbagana_pubkey::pubkey!("GK8R4uUmrawcREZ5xJy5dAzVV5V7aFvYg77id37pVTK"),
        gorbagana_pubkey::pubkey!("4vuWt1oHRqLMhf8Nv1zyEXZsYaeK7dipwrfKLoYU9Riq"),
        gorbagana_pubkey::pubkey!("EMhn1U3TMimW3bvWYbPUvN2eZnCfsuBN4LGWhzzYhiWR"),
        gorbagana_pubkey::pubkey!("BsKsunvENxAraBrL77UfAn1Gi7unVEmQAdCbhsjUN6tU"),
        gorbagana_pubkey::pubkey!("CTvhdUVf8KNyMbyEdnvRrBCHJjBKtQwkbj6zwoqcEssG"),
        gorbagana_pubkey::pubkey!("3fV2GaDKa3pZxyDcpMh5Vrh2FVAMUiWUKbYmnBFv8As3"),
        gorbagana_pubkey::pubkey!("4pV47TiPzZ7SSBPHmgUvSLmH9mMSe8tjyPhQZGbi1zPC"),
        gorbagana_pubkey::pubkey!("P8aKfWQPeRnsZtpBrwWTYzyAoRk74KMz56xc6NEpC4J"),
        gorbagana_pubkey::pubkey!("HuqDWJodFhAEWh6aWdsDVUqsjRket5DYXMYyDYtD8hdN"),
        gorbagana_pubkey::pubkey!("Ab1UcdsFXZVnkSt1Z3vcYU65GQk5MvCbs54SviaiaqHb"),
        gorbagana_pubkey::pubkey!("Dc2oHxFXQaC2QfLStuU7txtD3U5HZ82MrCSGDooWjbsv"),
        gorbagana_pubkey::pubkey!("3iPvAS4xdhYr6SkhVDHCLr7tJjMAFK4wvvHWJxFQVg15"),
        gorbagana_pubkey::pubkey!("GmyW1nqYcrw7P7JqrcyP9ivU9hYNbrgZ1r5SYJJH41Fs"),
        gorbagana_pubkey::pubkey!("E8jcgWvrvV7rwYHJThwfiBeQ8VAH4FgNEEMG9aAuCMAq"),
        gorbagana_pubkey::pubkey!("CY7X5o3Wi2eQhTocLmUS6JSWyx1NinBfW7AXRrkRCpi8"),
        gorbagana_pubkey::pubkey!("HQJtLqvEGGxgNYfRXUurfxV8E1swvCnsbC3456ik27HY"),
        gorbagana_pubkey::pubkey!("9xbcBZoGYFnfJZe81EDuDYKUm8xGkjzW8z4EgnVhNvsv"),
    ]
    .into()
}

// Withdraw authority for autostaked accounts on mainnet-beta
pub fn withdraw_authority() -> Vec<Pubkey> {
    [
        gorbagana_pubkey::pubkey!("8CUUMKYNGxdgYio5CLHRHyzMEhhVRMcqefgE6dLqnVRK"),
        gorbagana_pubkey::pubkey!("3FFaheyqtyAXZSYxDzsr5CVKvJuvZD1WE1VEsBtDbRqB"),
        gorbagana_pubkey::pubkey!("FdGYQdiRky8NZzN9wZtczTBcWLYYRXrJ3LMDhqDPn5rM"),
        gorbagana_pubkey::pubkey!("4e6KwQpyzGQPfgVr5Jn3g5jLjbXB4pKPa2jRLohEb1QA"),
        gorbagana_pubkey::pubkey!("FjiEiVKyMGzSLpqoB27QypukUfyWHrwzPcGNtopzZVdh"),
        gorbagana_pubkey::pubkey!("DwbVjia1mYeSGoJipzhaf4L5hfer2DJ1Ys681VzQm5YY"),
        gorbagana_pubkey::pubkey!("GeMGyvsTEsANVvcT5cme65Xq5MVU8fVVzMQ13KAZFNS2"),
        gorbagana_pubkey::pubkey!("Bj3aQ2oFnZYfNR1njzRjmWizzuhvfcYLckh76cqsbuBM"),
        gorbagana_pubkey::pubkey!("4ZJhPQAgUseCsWhKvJLTmmRRUV74fdoTpQLNfKoekbPY"),
        gorbagana_pubkey::pubkey!("HXdYQ5gixrY2H6Y9gqsD8kPM2JQKSaRiohDQtLbZkRWE"),
    ]
    .into()
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::genesis_utils::genesis_sysvar_and_builtin_program_lamports,
        gorbagana_account::{Account, AccountSharedData},
        gorbagana_epoch_schedule::EpochSchedule,
        gorbagana_genesis_config::{ClusterType, GenesisConfig},
        gorbagana_stake_interface::state::{Authorized, Lockup, Meta},
        std::{collections::BTreeMap, sync::Arc},
    };

    fn new_from_parent(parent: Arc<Bank>) -> Bank {
        let slot = parent.slot() + 1;
        let collector_id = Pubkey::default();
        Bank::new_from_parent(parent, &collector_id, slot)
    }

    #[test]
    fn test_calculate_non_circulating_supply() {
        let mut accounts: BTreeMap<Pubkey, Account> = BTreeMap::new();
        let balance = 10;
        let num_genesis_accounts = 10;
        for _ in 0..num_genesis_accounts {
            accounts.insert(
                gorbagana_pubkey::new_rand(),
                Account::new(balance, 0, &Pubkey::default()),
            );
        }
        let non_circulating_accounts = non_circulating_accounts();
        let num_non_circulating_accounts = non_circulating_accounts.len() as u64;
        for key in non_circulating_accounts.clone() {
            accounts.insert(key, Account::new(balance, 0, &Pubkey::default()));
        }

        let num_stake_accounts = 3;
        for _ in 0..num_stake_accounts {
            let pubkey = gorbagana_pubkey::new_rand();
            let meta = Meta {
                authorized: Authorized::auto(&pubkey),
                lockup: Lockup {
                    epoch: 1,
                    ..Lockup::default()
                },
                ..Meta::default()
            };
            let stake_account = Account::new_data_with_space(
                balance,
                &StakeStateV2::Initialized(meta),
                StakeStateV2::size_of(),
                &stake::program::id(),
            )
            .unwrap();
            accounts.insert(pubkey, stake_account);
        }

        let slots_per_epoch = 32;
        let genesis_config = GenesisConfig {
            accounts,
            epoch_schedule: EpochSchedule::new(slots_per_epoch),
            cluster_type: ClusterType::MainnetBeta,
            ..GenesisConfig::default()
        };
        let mut bank = Arc::new(Bank::new_for_tests(&genesis_config));
        assert_eq!(
            bank.capitalization(),
            (num_genesis_accounts + num_non_circulating_accounts + num_stake_accounts) * balance
                + genesis_sysvar_and_builtin_program_lamports(),
        );

        let non_circulating_supply = calculate_non_circulating_supply(&bank).unwrap();
        assert_eq!(
            non_circulating_supply.lamports,
            (num_non_circulating_accounts + num_stake_accounts) * balance
        );
        assert_eq!(
            non_circulating_supply.accounts.len(),
            num_non_circulating_accounts as usize + num_stake_accounts as usize
        );

        bank = Arc::new(new_from_parent(bank));
        let new_balance = 11;
        for key in non_circulating_accounts {
            bank.store_account(
                &key,
                &AccountSharedData::new(new_balance, 0, &Pubkey::default()),
            );
        }
        let non_circulating_supply = calculate_non_circulating_supply(&bank).unwrap();
        assert_eq!(
            non_circulating_supply.lamports,
            (num_non_circulating_accounts * new_balance) + (num_stake_accounts * balance)
        );
        assert_eq!(
            non_circulating_supply.accounts.len(),
            num_non_circulating_accounts as usize + num_stake_accounts as usize
        );

        // Advance bank an epoch, which should unlock stakes
        for _ in 0..slots_per_epoch {
            bank = Arc::new(new_from_parent(bank));
        }
        assert_eq!(bank.epoch(), 1);
        let non_circulating_supply = calculate_non_circulating_supply(&bank).unwrap();
        assert_eq!(
            non_circulating_supply.lamports,
            num_non_circulating_accounts * new_balance
        );
        assert_eq!(
            non_circulating_supply.accounts.len(),
            num_non_circulating_accounts as usize
        );
    }
}
