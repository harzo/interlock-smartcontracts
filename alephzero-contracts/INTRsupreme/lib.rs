// INTERLOCK NETWORK
//
// blairmunroakusa@0653Tue.28Jun22.anch.AK:br
//
// THIS IS A TEST SUPREME NODE CONTRACT TO
// INTEGRATE TOKEN AND STAKING CONTRACTS
// USING INK! FRAMEWORK



// !!!!! INCOMPLETE AND FLAWED, WARNING !!!!!




#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod intrsupreme {

    use intrtoken::INTRtokenRef;
    use intrstake::INTRstakeRef;
    use stakedata::StakeDataRef;
    
    use ink_storage::traits::{
        PackedLayout,
        SpreadLayout,
    };
    use ink_prelude::string::String;

    #[ink(storage)]
    pub struct INTRsupreme {
        intrtoken: INTRtokenRef,
        intrstake: INTRstakeRef,
        stakedata: StakeDataRef,
    }

    impl INTRsupreme {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new_supreme(
            init_value: u32,
            version: u32,
            token_code_hash: Hash,
            stake_code_hash: Hash,
            stakedata_code_hash: Hash,
        ) -> Self {
            let total_balance = Self::env().balance();
            let salt = version.to_le_bytes();
            let intrtoken = INTRtokenRef::new_token(init_value)
                .endowment(total_balance/4)
                .code_hash(token_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to instantiate token contract: {:?}", error)
                });
            let stakedata = StakeDataRef::new()
                .endowment(total_balance/4)
                .code_hash(stakedata_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to instantiate stakedata contract: {:?}", error)
                });
            let intrstake = INTRstakeRef::new(intrtoken.clone(), stakedata.clone())
                .endowment(total_balance/4)
                .code_hash(stake_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to instantiate stake contract: {:?}", error)
                });


            Self {
                intrtoken,
                intrstake,
                stakedata,
            }
        }


// ERC20 methods
        
        /// name getter
        #[ink(message)]
        pub fn name(&self) -> String {
            self.intrtoken.name()
        }

        /// symbol getter
        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.intrtoken.symbol()
        }

        /// decimals getter
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.intrtoken.decimals()
        }

        /// total supply getter
        #[ink(message)]
        pub fn total_supply(&self) -> u32 {
            self.intrtoken.total_supply()
        }

        /// account balance getter
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u32 {
            self.intrtoken.balance_of(account)
        }

        /// account allowance getter
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u32 {
            self.intrtoken.allowance(owner, spender)
        }

        /// transfer token
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u32) -> bool {
            self.intrtoken.transfer(to, amount)
        }

        /// transfer token from, to
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: u32) -> bool {
            self.intrtoken.transfer_from(from, to, amount)
        }

        /// approve token spending
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: u32) -> bool {
            self.intrtoken.approve(spender, amount)
        }

    }


}
