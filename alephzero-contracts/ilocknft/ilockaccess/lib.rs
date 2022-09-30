//
// INTERLOCK NETWORK - PSP34 ACCESS CONTRACT (No. 1)
//
// INCLUDES:
// - BOUNCER LICENSE NFT CLASS
// - VIP MEMBERSHIP NFT CLASS
// - ...
//
// !!!!! INCOMPLETE AND UNAUDITED, WARNING !!!!!
//
// This is a standard ERC721-style token contract
// with provisions for enforcing proof of Bouncer
// NFT license ownership, proof of VIP membership,
// and other access features in future upgrades.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod ilockaccess1 {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };
    use ink_prelude::string::String;
    use openbrush::{
        contracts::{
            psp34::extensions::{
                metadata::*,
                mintable::*,
            },
            ownable::*,
        },
        traits::Storage,
    };

    /// . ACCESS_CLASS is metadata attribute id
    /// . BOUNCER_LICENSE and VIP_MEMBERSHIP are attributes of ACCESS_CLASS
    pub const ACCESS_CLASS: &str = "ACCESS_CLASS";
    pub const BOUNCER_LICENSE: &str = "BOUNCER_LICENSE";
    pub const VIP_MEMBERSHIP: &str = "VIP_MEMBERSHIP";

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct ILOCKaccess1 {
        #[storage_field]
        psp34: psp34::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        ownable: ownable::Data,
        next_bouncerlicense_id: u32,
        next_vipmembership_id: u32,
        authenticated: Mapping<(AccountId, u32), bool>,
    }

    impl PSP34          for ILOCKaccess1 {}
    impl PSP34Metadata  for ILOCKaccess1 {}
    impl Ownable        for ILOCKaccess1 {}
    impl PSP34Mintable  for ILOCKaccess1 {
        
        /// . mint general NFT
        /// . overrides extention mint() to enforce only_owner modifier
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        fn mint(&mut self, recipient: AccountId, id: Id) -> Result<(), PSP34Error> {

            self._mint_to(recipient, id)?;

            Ok(())
        }
    }

    impl ILOCKaccess1 {

        #[ink(constructor)]
        pub fn new(
        ) -> Self {

            ink_lang::codegen::initialize_contract(|contract: &mut Self| {
                
                contract._init_with_owner(contract.env().caller());
                contract.next_bouncerlicense_id = 0;
                contract.next_vipmembership_id = 10_000;

				let collection_id = contract.collection_id();
				contract._set_attribute(
                    collection_id.clone(),
                    String::from("name").into_bytes(),
                    String::from("Interlock Access NFTs").into_bytes(),
                );
				contract._set_attribute(
                    collection_id,
                    String::from("symbol").into_bytes(),
                    String::from("ILOCKACCESS").into_bytes(),
                );
            })
        }

        /// . mint an NFT Bouncer license
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn mint_bouncerlicense(&mut self, recipient: AccountId) -> Result<(), PSP34Error> {

            self._mint_to(recipient, psp34::Id::U32(self.next_bouncerlicense_id))?;
            self._set_attribute(
                psp34::Id::U32(self.next_bouncerlicense_id),
                ACCESS_CLASS.as_bytes().to_vec(),
                BOUNCER_LICENSE.as_bytes().to_vec(),
            );
            self.next_bouncerlicense_id += 1;

            Ok(())
        }

        /// . mint an NFT VIP membership certificate
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn mint_vipmembership(&mut self, recipient: AccountId) -> Result<(), PSP34Error> {

            self._mint_to(recipient, psp34::Id::U32(self.next_vipmembership_id))?;
            self._set_attribute(
                psp34::Id::U32(self.next_vipmembership_id),
                ACCESS_CLASS.as_bytes().to_vec(),
                VIP_MEMBERSHIP.as_bytes().to_vec(),
            );
            self.next_vipmembership_id += 1;

            Ok(())
        }

        /// . grant 'authenticated' status to interlocker
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn set_authenticated(&mut self, holder: AccountId, id: u32) -> Result<(), PSP34Error> {

            self.authenticated.insert((holder, id), &true);

            Ok(())
        }

        /// . revoke 'authenticated' status from interlocker
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn set_not_authenticated(&mut self, holder: AccountId, id: u32) -> Result<(), PSP34Error> {

            self.authenticated.insert((holder, id), &false);

            Ok(())
        }

        /// . get authentication status for NFT
        #[ink(message)]
        pub fn authentication_status(&mut self, holder: AccountId, id: u32) -> Result<bool, PSP34Error> {

            Ok(self.authenticated.get((holder, id)).unwrap())
        }

        /// . modifies the code which is used to execute calls to this contract address
        /// . this upgrades the token contract logic while using old state
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn upgrade_contract(
            &mut self,
            code_hash: [u8; 32]
        ) -> Result<(), PSP34Error> {

            // takes code hash of updates contract and modifies preexisting logic to match
            ink_env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });

            Ok(())
        }
    }
}
