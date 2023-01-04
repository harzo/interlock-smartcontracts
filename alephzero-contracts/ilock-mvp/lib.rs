//
// INTERLOCK NETWORK MVP SMART CONTRACTS
//  - PSP22 TOKEN
//  - REWARDS
//
// !!!!! INCOMPLETE AND UNAUDITED, WARNING !!!!!
//
// This is a standard ERC20-style token contract
// with provisions for enforcing a token distribution
// vesting schedule, and for rewarding interlockers for
// browsing the internet with the Interlock browser extension.
//
// This contract build may need to be done after running
//
//      cargo install cargo-contract --version 2.0.0-beta
//
// The contract may be built running
//
//      cargo contract build
//


#![allow(non_snake_case)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

pub use self::ilocktoken::{
    ILOCKtoken,
    ILOCKtokenRef,
};

#[openbrush::contract]
pub mod ilocktoken {

    use ink::{
        codegen::{EmitEvent, Env},
        reflect::ContractEventBase,
    };
    use ink::prelude::{
        vec::Vec,
        format,
        string::{String, ToString},
    };
    use ink::storage::Mapping;
    use openbrush::{
        contracts::{
            psp22::{
                extensions::{metadata::*, burnable::*},
                Internal,
            },
            ownable::*,
        },
        traits::Storage,
    };

////////////////////////////////////////////////////////////////////////////
//// constants /////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

    /// . magic numbers
    pub const ID_LENGTH: usize = 32;                                // 32B account id
    pub const POOL_COUNT: usize = 12;                               // number of stakeholder pools
    pub const ONE_MONTH: Timestamp = 2_592_000_000;                 // milliseconds in 30 days

    /// . token data
    pub const TOKEN_CAP: u128 = 1_000_000_000;                      // 10^9
    pub const DECIMALS_POWER10: u128 = 1_000_000_000_000_000_000;   // 10^18
    pub const SUPPLY_CAP: u128 = TOKEN_CAP * DECIMALS_POWER10;      // 10^27
    pub const TOKEN_NAME: &str = "Interlock Network";
    pub const TOKEN_DECIMALS: u8 = 18;
    pub const TOKEN_SYMBOL: &str = "ILOCK";

    #[derive(Debug)]
    pub struct PoolData<'a> {
        name: &'a str,
        tokens: u128,
        vests: u8,
        cliffs: u8,
    }

    /// . pool data
    pub const POOLS: [PoolData; POOL_COUNT] = [
        PoolData { name: "early_backers+venture_capital", tokens: 20_000_00,   vests: 24, cliffs: 1, },
        PoolData { name: "presale_1",                     tokens: 48_622_222,  vests: 18, cliffs: 1, },
        PoolData { name: "presale_2",                     tokens: 66_666_667,  vests: 15, cliffs: 1, },
        PoolData { name: "presale_3",                     tokens: 40_000_000,  vests: 12, cliffs: 1, },
        PoolData { name: "team+founders",                 tokens: 200_000_000, vests: 36, cliffs: 6, },
        PoolData { name: "outlier_ventures",              tokens: 40_000_000,  vests: 24, cliffs: 1, },
        PoolData { name: "advisors",                      tokens: 25_000_000,  vests: 24, cliffs: 1, },
        PoolData { name: "rewards",                       tokens: 285_000_000, vests: 1,  cliffs: 0, },
        PoolData { name: "foundation",                    tokens: 172_711_111, vests: 84, cliffs: 1, },
        PoolData { name: "partners",                      tokens: 37_000_000,  vests: 1,  cliffs: 0, },
        PoolData { name: "whitelist",                     tokens: 15_000_000,  vests: 48, cliffs: 0, },
        PoolData { name: "public_sale",                   tokens: 50_000_000,  vests: 48, cliffs: 0, },
    ];

    pub const EARLY_BACKERS: u8     = 0;
    pub const PRESALE_1: u8         = 1;
    pub const PRESALE_2: u8         = 2;
    pub const PRESALE_3: u8         = 3;
    pub const TEAM_FOUNDERS: u8     = 4;
    pub const OUTLIER_VENTURES: u8  = 5;
    pub const ADVISORS: u8          = 6;
    pub const REWARDS: u8           = 7;
    pub const FOUNDATION: u8        = 8;
    pub const PARTNERS: u8          = 9;
    pub const WHITELIST: u8         = 10;
    pub const PUBLIC_SALE: u8       = 11;

////////////////////////////////////////////////////////////////////////////
//// structured data ///////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

    /// . StakeholderData struct contains all pertinent information for each stakeholder
    ///   (Besides balance and allowance mappings)
    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
    )]
    #[derive(Debug)]
    pub struct StakeholderData {
        paid: Balance,
        share: Balance,
        pool: u8,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Default)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
    )]
    pub struct Port {
        hash: Hash,
        tax: Balance,
        cap: Balance,
        locked: bool,
        paid: Balance,
        collected: Balance,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Default)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
    )]
    pub struct Socket {
        address: AccountId,
        port: u16,
    }

    /// . ILOCKtoken struct contains overall storage data for contract
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ILOCKtoken {

        // ABSOLUTELY DO NOT CHANGE THE ORDER OF THESE VARIABLES, OR TYPE!!!
        // . TO ADD NEW VARIABLE, IT MUST BE APPENDED TO END OF LIST

        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
		ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,

        stakeholderdata: Mapping<AccountId, StakeholderData>,
        rewardedinterlocker: Mapping<AccountId, Balance>,
        poolbalances: [Balance; POOL_COUNT],
        rewardedtotal: Balance,
        circulatingsupply: Balance,
        taxpool: Balance,
        monthspassed: u8,
        nextpayout: Timestamp,
        ports: Mapping<u16, Port>,        // port -> (hash of port contract, tax)
        sockets: Mapping<AccountId, Socket>,  // contract address -> socket
                                                        // socket == owneraddress:port
        // ADD NEW VARIABLES BELOW FOR
        // ANY SOCKET LOGIC CODE HASH UPDATE
        // VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV

        // newvariable1: Var1,
        // ...
        
        // ONLY APPEND NEW VARIABLES TO END OF
        // PREEXISTING VARIABLES
    }

////////////////////////////////////////////////////////////////////////////
//// events and errors /////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

    /// . specify transfer event
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        amount: Balance,
    }

    /// . specify approve event
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: Option<AccountId>,
        #[ink(topic)]
        spender: Option<AccountId>,
        amount: Balance,
    }

    /// . specify reward event
    #[ink(event)]
    pub struct Reward {
        #[ink(topic)]
        to: Option<AccountId>,
        amount: Balance,
    }

    /// . Other contract error types
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum OtherError {
        /// Returned if caller is not contract owner
        CallerNotOwner,
        /// Returned if stakeholder share is entirely paid out
        StakeholderSharePaid,
        /// Returned if the stakeholder doesn't exist
        StakeholderNotFound,
        /// Returned if stakeholder has not yet passed cliff
        CliffNotPassed,
        /// Returned if it is too soon to payout for month
        PayoutTooEarly,
        /// Returned if reward is too large
        PaymentTooLarge,
        /// Returned if socket does not exist
        NoSocket,
        /// Returned if port does not exist
        NoPort,
        /// Returned if not contract
        NotContract,
        /// Returned if only owner can add socket
        PortLocked,
        /// Returned if port cap is surpassed
        PortCapSurpassed,
        /// Returned if reward recipient is a contract
        CannotRewardContract,
        /// Returned if socket contract does not match registered hash
        UnsafeContract,
        /// Returned if socket contract does not match registered hash
        Custom(String),
    }

    impl Into<PSP22Error> for OtherError {
        fn into(self) -> PSP22Error {
            PSP22Error::Custom(format!("{:?}", self).into_bytes())
        }
    }

    impl Into<OtherError> for PSP22Error {
        fn into(self) -> OtherError {
            OtherError::Custom(format!("{:?}", self))
        }
    }

    pub type PSP22Result<T> = core::result::Result<T, PSP22Error>;

    /// . OtherError result type.
    pub type OtherResult<T> = core::result::Result<T, OtherError>;

    pub type Event = <ILOCKtoken as ContractEventBase>::Type;

////////////////////////////////////////////////////////////////////////////
/////// reimplement some functions /////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

    impl PSP22 for ILOCKtoken {
        
        /// . override default total_supply getter
        /// . total supply reflects token in circulation
        #[ink(message)]
        fn total_supply(&self) -> Balance {

            self.circulatingsupply
        }

        /// . override default transfer doer
        /// . transfer from owner increases total supply
        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {

            let from = self.env().caller();

            let _ = self._transfer_from_to(from, to, value, data)?;

            // if sender is owner, then tokens are entering circulation
            if from == self.ownable.owner {
                self.circulatingsupply += value;
            }

            // if recipient is owner, then tokens are being returned or added to rewards pool
            if to == self.ownable.owner {
                self.poolbalances[REWARDS as usize] += value;
                self.circulatingsupply -= value;
            }

            Ok(())
        }

        /// . override default transfer_from_to doer
        /// . transfer from owner increases total supply
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {

            let _ = self._transfer_from_to(from, to, value, data)?;

            // if sender is owner, then tokens are entering circulation
            if from == self.ownable.owner {
                self.circulatingsupply += value;
            }

            // if recipient is owner, then tokens are being returned or added to rewards pool
            if to == self.ownable.owner {
                self.poolbalances[REWARDS as usize] += value;
                self.circulatingsupply -= value;
            }

            Ok(())
        }

    }

    impl PSP22Metadata for ILOCKtoken {}

    impl Ownable for ILOCKtoken {
        
        // PRIOR TO OWNER TRANSFER,
        // REMAINING OWNER NONCIRCULATING
        // BALANCE MUST BE TRANSFERRED TO NEW OWNER.
    }

    impl PSP22Burnable for ILOCKtoken {

        /// . override default burn doer
        /// . burn function to permanently remove tokens from circulation / supply
        #[ink(message)]
		#[openbrush::modifiers(only_owner)]
        fn burn(
            &mut self,
            donor: AccountId,
            amount: Balance,
        ) -> PSP22Result<()> {

            // burn the tokens
            let _ = self._burn_from(donor, amount)?;
            self.circulatingsupply -= amount;

            Ok(())
        }
	}

    // these implementations are because open brush does not implement
    impl Internal for ILOCKtoken {

        fn _emit_transfer_event(
            &self,
            _from: Option<AccountId>,
            _to: Option<AccountId>,
            _amount: Balance,
        ) {
            ILOCKtoken::emit_event(
                self.env(),
                Event::Transfer(Transfer {
                    from: _from,
                    to: _to,
                    amount: _amount,
                }),
            );
        }

        fn _emit_approval_event(
            &self,
            _owner: AccountId,
            _spender: AccountId,
            _amount: Balance
        ) {
            ILOCKtoken::emit_event(
                self.env(),
                Event::Approval(Approval {
                    owner: Some(_owner),
                    spender: Some(_spender),
                    amount: _amount,
                }),
            );
        }
    }

////////////////////////////////////////////////////////////////////////////
/////// implement token contract ///////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

    impl ILOCKtoken {

        // Pete said this was probably necessary
        /// . function for internal _emit_event implementations
        pub fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }

        /// . constructor to initialize contract
        /// . note: pool contracts must be created prior to construction (for args)
        #[ink(constructor)]
        pub fn new_token(
        ) -> Self {

            // create contract
            let mut contract = Self::default();
//            ink_lang::codegen::initialize_contract(|contract: &mut Self| {
                
            // define owner as caller
            let caller = contract.env().caller();

            // set initial data
            contract.monthspassed = 0;
            contract.nextpayout = Self::env().block_timestamp() + ONE_MONTH;
            contract.rewardedtotal = 0;
            contract.circulatingsupply = 0;

            contract.metadata.name = Some(TOKEN_NAME.to_string().into_bytes());
            contract.metadata.symbol = Some(TOKEN_SYMBOL.to_string().into_bytes());
            contract.metadata.decimals = TOKEN_DECIMALS;

            // mint with openbrush:
            contract._mint_to(caller, SUPPLY_CAP)
                    .expect("Failed to mint the initial supply");
            contract._init_with_owner(caller);

            // create initial pool balances
            for pool in 0..POOL_COUNT {

                contract.poolbalances[pool] =
                                POOLS[pool].tokens * DECIMALS_POWER10;
            }
            
            contract
        }

////////////////////////////////////////////////////////////////////////////
/////// timing /////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . function to check if enough time has passed to collect next payout
        /// . this function ensures Interlock cannot rush the vesting schedule
        /// . this function must be called before the next round of token distributions
        #[ink(message)]
        pub fn check_time(
            &mut self,
        ) -> OtherResult<()> {

            // test to see if current time falls beyond time for next payout
            if self.env().block_timestamp() > self.nextpayout {

                // update time variables
                self.nextpayout += ONE_MONTH;
                self.monthspassed += 1;

                return Ok(());
            }

            // too early, do nothing
            return Err(OtherError::PayoutTooEarly)
        }
        
        /// . time in seconds until next payout in minutes
        #[ink(message)]
        pub fn remaining_time_until_next_payout(
            &self
        ) -> Timestamp {

            // add logic here to return 0 if overflow

            (self.nextpayout - self.env().block_timestamp()) / 60_000
        }

////////////////////////////////////////////////////////////////////////////
/////// stakeholders  //////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . function that registers a stakeholder's wallet and vesting info
        /// . used to calculate monthly payouts and track net paid
        /// . stakeholder data also used for stakeholder to verify their place in vesting schedule
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn register_stakeholder(
            &mut self,
            stakeholder: AccountId,
            share: Balance,
            pool: u8,
        ) -> PSP22Result<()> {

            // create stakeholder struct
            let this_stakeholder = StakeholderData {
                paid: 0,
                share: share,
                pool: pool,
            };

            // insert stakeholder struct into mapping
            self.stakeholderdata.insert(stakeholder, &this_stakeholder);

            Ok(())
        }

        /// . function that returns a stakeholder's payout and other data
        /// . this will allow stakeholders to verify their stake from explorer if so motivated
        /// . returns tuple (paidout, payremaining, payamount, poolnumber)
        #[ink(message)]
        pub fn stakeholder_data(
            &self,
            stakeholder: AccountId,
        ) -> (String, String, String, String) {

            // get pool and stakeholder data structs first
            let this_stakeholder = self.stakeholderdata.get(stakeholder).unwrap();
            let pool = &POOLS[this_stakeholder.pool as usize];

            // how much has stakeholder already claimed?
            let paidout: Balance = this_stakeholder.paid;

            // how much does stakeholder have yet to collect?
            let payremaining: Balance = this_stakeholder.share - this_stakeholder.paid;

            // how much does stakeholder get each month?
            let payamount: Balance = this_stakeholder.share / pool.vests as Balance;

            return (
                format!("paidout: {:?} ", paidout),
                format!("payremaining: {:?} ", payremaining),
                format!("payamount: {:?} ", payamount),
                format!("pool: {:?}", POOLS[this_stakeholder.pool as usize].name),
            )
        }

////////////////////////////////////////////////////////////////////////////
/////// token distribution /////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . general function to transfer the token share a stakeholder is currently entitled to
        /// . this is called once per stakeholder by Interlock, Interlock paying fees
        /// . pools are guaranteed to have enough tokens for all stakeholders
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn distribute_tokens(
            &mut self,
            stakeholder: AccountId,
        ) -> PSP22Result<()> {

            // get data structs
            let mut this_stakeholder = match self.stakeholderdata.get(stakeholder) {
                Some(s) => s,
                None => { return Err(OtherError::StakeholderNotFound.into()) },
            };
            let pool = &POOLS[this_stakeholder.pool as usize];

            // require cliff to have been surpassed
            if self.monthspassed < pool.cliffs {
                return Err(OtherError::CliffNotPassed.into())
            }

            // require share has not been completely paid out
            if this_stakeholder.paid == this_stakeholder.share {
                return Err(OtherError::StakeholderSharePaid.into())
            }

            // calculate the payout owed
            let mut payout: Balance = this_stakeholder.share / pool.vests as Balance;

            // require that payout isn't repeatable for this month
            let payments = this_stakeholder.paid / payout;
            if payments >= self.monthspassed as u128 {
                return Err(OtherError::PayoutTooEarly.into())
            }

            // if this is final payment, add token remainder to payout
            // (this is to compensate for floor division that calculates payamount)
            if this_stakeholder.share - this_stakeholder.paid - payout <
                this_stakeholder.share / pool.vests as Balance {

                // add remainder
                payout += this_stakeholder.share % pool.vests as Balance;
            }

            // now transfer tokens
            let _ = self.transfer(stakeholder, payout, Default::default())?;

            // update pool balance
            self.poolbalances[this_stakeholder.pool as usize] -= payout;

            // finally update stakeholder data struct state
            this_stakeholder.paid += payout;
            self.stakeholderdata.insert(stakeholder, &this_stakeholder);

            Ok(())
        }

        /// . function used to distribute tokens to whitelisters
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn distribute_whitelist(
            &mut self,
            stakeholder: AccountId,
            amount: Balance,
        ) -> PSP22Result<()> {

            // make sure reward not too large
            if self.poolbalances[WHITELIST as usize] < amount {
                return Err(OtherError::PaymentTooLarge.into())
            }

            // decrement pool balance
            self.poolbalances[WHITELIST as usize] -= amount;

            // now transfer tokens
            let _ = self.transfer(stakeholder, amount, Default::default())?;

            Ok(())
        }

        /// . function used to distribute tokens to public sale entities (ie exchanges)
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn distribute_publicsale(
            &mut self,
            stakeholder: AccountId,
            amount: Balance,
        ) -> PSP22Result<()> {

            // make sure reward not too large
            if self.poolbalances[PUBLIC_SALE as usize] < amount {
                return Err(OtherError::PaymentTooLarge.into())
            }

            // decrement pool balance
            self.poolbalances[PUBLIC_SALE as usize] -= amount;

            // now transfer tokens
            let _ = self.transfer(stakeholder, amount, Default::default())?;

            Ok(())
        }

        /// . function used to distribute tokens to strategic partners
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn distribute_partners(
            &mut self,
            stakeholder: AccountId,
            amount: Balance,
        ) -> PSP22Result<()> {

            // make sure reward not too large
            if self.poolbalances[PARTNERS as usize] < amount {
                return Err(OtherError::PaymentTooLarge.into())
            }

            // decrement pool balance
            self.poolbalances[PARTNERS as usize] -= amount;

            // now transfer tokens
            let _ = self.transfer(stakeholder, amount, Default::default())?;

            Ok(())
        }

////////////////////////////////////////////////////////////////////////////
/////// pool data //////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . function that returns pool data
        /// . this will allow observers to verify vesting parameters for each pool (esp. theirs)
        /// . observers may verify pool data from explorer if so motivated
        /// . pool numbers range from 0-11
        /// . returns (name, tokens, vests, cliff)
        #[ink(message)]
        pub fn pool_data(
            &self,
            pool: u8,
        ) -> (String, String, String, String) {
        
            let pool = &POOLS[pool as usize];
            // just grab up and send it out
            return (
                format!("pool: {:?} ", pool.name.to_string()),
                format!("tokens alotted: {:?} ", pool.tokens),
                format!("number of vests: {:?} ", pool.vests),
                format!("vesting cliff: {:?} ", pool.cliffs),
            )
        }
        
        /// . get current balance of whitelist pool
        #[ink(message)]
        pub fn pool_balance(
            &self,
            pool: u8,
        ) -> (String, Balance) {

            (format!("pool: {:?} balance: {:?}", 
                    POOLS[pool as usize].name.to_string(),
                    self.poolbalances[pool as usize]),
             self.poolbalances[pool as usize])
        }

////////////////////////////////////////////////////////////////////////////
//// rewarding  ////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . reward the interlocker for browsing
        /// . this is a manual rewarding function, to override the socket formalism
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn reward_interlocker(
            &mut self,
            reward: Balance,
            interlocker: AccountId
        ) -> PSP22Result<Balance> {

            // make sure reward not too large
            if self.poolbalances[REWARDS as usize] < reward {
                return Err(OtherError::PaymentTooLarge.into())
            }

            // update total amount rewarded to interlocker
            self.rewardedtotal += reward;

            // update rewards pool balance
            self.poolbalances[REWARDS as usize] -= reward;

            // transfer reward tokens from rewards pool to interlocker
            let _ = self.transfer(interlocker, reward, Default::default())?;

            // get previous total rewarded to interlocker
            let rewardedinterlockertotal: Balance = match self.rewardedinterlocker.get(interlocker) {
                Some(total) => total,
                None => 0,
            };
            self.rewardedinterlocker.insert(interlocker, &(rewardedinterlockertotal + reward));

            // emit Reward event
            self.env().emit_event(Reward {
                to: Some(interlocker),
                amount: reward,
            });

            // this returns interlocker total reward amount for extension display purposes
            Ok(rewardedinterlockertotal + reward)
        }

        /// . get amount rewarded to interlocker to date
        #[ink(message)]
        pub fn rewarded_interlocker_total(
            &self,
            interlocker: AccountId
        ) -> Balance {

            match self.rewardedinterlocker.get(interlocker) {
                Some(total) => total,
                None => 0,
            }
        }

        /// . get total amount rewarded to date
        #[ink(message)]
        pub fn rewarded_total(
            &self
        ) -> Balance {

            self.rewardedtotal
        }

////////////////////////////////////////////////////////////////////////////
//// misc  /////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
        
        /// . get current balance of whitelist pool
        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn withdraw_tax(
            &mut self,
            wallet: AccountId,
            amount: Balance
        ) -> PSP22Result<()> {

            // only withdraw what is available in pool
            if amount > self.taxpool {

                return Err(OtherError::PaymentTooLarge.into());
            }

            let _ = self.transfer(wallet, amount, Default::default())?;

            self.taxpool -= amount;

            Ok(())
        }

        /// . display taxpool balance
        #[ink(message)]
        pub fn tax_available(
            &self,
        ) -> Balance {

            self.taxpool
        }

        /// . function to get the number of months passed for contract
        #[ink(message)]
        pub fn months_passed(
            &self,
        ) -> u8 {

            self.monthspassed
        }

        /// . function to get the supply cap minted on TGE
        #[ink(message)]
        pub fn cap(
            &self,
        ) -> u128 {

            SUPPLY_CAP
        }

        /// . function to increment monthspassed for testing
        #[ink(message)]
        pub fn TESTING_increment_month(
            &mut self,
        ) -> bool {

            self.monthspassed += 1;

            true
        }

////////////////////////////////////////////////////////////////////////////
//// portability and extensibility  ////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

        /// . modifies the code which is used to execute calls to this contract address
        /// . this upgrades the token contract logic while using old state
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn update_contract(
            &mut self,
            code_hash: [u8; 32]
        ) -> PSP22Result<()> {

            // takes code hash of updates contract and modifies preexisting logic to match
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });

            Ok(())
        }

        /// . rewards/staking contracts register with token contract here
        /// . contract must first register with token contract to allow reward transfers
        #[ink(message)]
        pub fn create_socket(
            &mut self,
            owner: AccountId,
            number: u16,
        ) -> OtherResult<()> {

            // make sure caller is a contact, return if not
            if !self.env().is_contract(&self.env().caller()) {

                return Err(OtherError::NotContract);
            };

            // get hash of calling contract
            let calling_hash: Hash = match self.env().code_hash(&self.env().caller()) {
                Ok(hash) => hash,
                Err(_) => return Err(OtherError::NotContract),
            };

            // get port specified by calling contract
            let port: Port = match self.ports.get(number) {
                Some(port) => port,
                None => return Err(OtherError::NoPort),
            };

            // make sure port is unlocked, or caller is token contract owner
            //   . this makes it so that people can't build their own client application
            //     to 'hijack' an approved and registered rewards contract.
            //   . if port is locked then only interlock can register new reward contract
            if port.locked && (self.ownable.owner != owner) {

                return Err(OtherError::PortLocked);
            }
            
            // compare calling contract hash to registered port hash
            // to make sure it is safe (ie, approved and audited by interlock)
            if calling_hash == port.hash {
                
                // if the same, contract is allowed to create socket
                let contract: AccountId = self.env().caller();
                let socket = Socket { address: owner, port: number };

                // socket is registered with token contract
                // and calling contract may start calling socket to receive rewards
                self.sockets.insert(contract, &socket);
            
                // give socket allowance up to port cap
                //   . connecting contracts will not be able to reward
                //     more than cap specified by interlock (for safety)
                self.psp22.allowances.insert(
                    &(&self.ownable.owner, &self.env().caller()),
                    &port.cap
                );

                self._emit_approval_event(self.ownable.owner, owner, port.cap);

                return Ok(()); 
            }

            // returns error if calling contract is not a known
            // safe contract registered by interlock as a 'port' that 
            // the calling contract can connect to
            Err(OtherError::UnsafeContract)
        }

        /// . check for socket and charge owner per port spec
        #[ink(message)]
        pub fn call_socket(
            &mut self,
            address: AccountId,
            amount: Balance,
        ) -> OtherResult<()> {

            // make sure address is not contract
            if self.env().is_contract(&address) {

                return Err(OtherError::CannotRewardContract);
            }

            // get socket, to get port assiciated with socket
            let socket: Socket = match self.sockets.get(self.env().caller()) {
                Some(socket) => socket,
                None => return Err(OtherError::NoSocket),
            };

            // port owner address
            let owner: AccountId = socket.address;


            // tax socket owner, inject port logic, transfer reward
            match socket.port {

                // NOTE: injecting custom logic into port requires Interlock Token
                //       contract codehash update after internal port contract audit
                
                // reserved Interlock ports
                0 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                1 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                2 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                3 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                4 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                5 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                6 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                7 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                8 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                9 => { self.tax_and_reward(owner, address, amount, socket.port)? },

                // reserved community node ports
                10 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                11 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                12 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                13 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                14 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                15 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                16 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                17 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                18 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                19 => { self.tax_and_reward(owner, address, amount, socket.port)? },

                20 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                21 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                22 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                23 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                24 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                25 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                26 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                27 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                28 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                29 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                
                30 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                31 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                32 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                33 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                34 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                35 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                36 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                37 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                38 => { self.tax_and_reward(owner, address, amount, socket.port)? },
                39 => { self.tax_and_reward(owner, address, amount, socket.port)? },

                // ... custom logic example:

                65535 => {

                    // < inject custom logic here >

                    // then reward and tax
                    self.tax_and_reward(owner, address, amount, socket.port)?
                },

                _ => return Err(OtherError::Custom(format!("Socket registered with invalid port."))),
            };

            Ok(())
        }

        /// . tax and reward socket
        pub fn tax_and_reward(
            &mut self,
            owner: AccountId,
            address: AccountId,
            amount: Balance,
            portnumber: u16,
        ) -> OtherResult<()> {

            // get port info
            let mut port: Port = match self.ports.get(portnumber) {
                Some(port) => port,
                None => return Err(OtherError::NoPort),
            };

            // make sure this will not exceed port cap
            if port.cap < (port.paid + amount) {

                return Err(OtherError::PortCapSurpassed.into());
            }

            // TODO/QUESTION:
            // tax should probably be a fraction of reward,
            // instead of a flat rate per reward
            //   . this would change the logic a little bit
            // ?

            // transfer transaction tax from socket owner to token contract owner
            let _ = match self.transfer_from(owner, self.ownable.owner, port.tax, Default::default()) {
                Err(error) => return Err(error.into()),
                Ok(()) => (),  
            };

            // update pools
            self.taxpool += port.tax;
            port.collected += port.tax;

            // transfer reward to reward recipient
            let _ = match self.transfer_from(self.ownable.owner, address, amount, Default::default()) {
                Err(error) => return Err(error.into()),
                Ok(()) => (),
            };

            // update balance pool and totals
            // (the port.tax subtraction is to offset rewardpool increase on transfer from token owner)
            self.poolbalances[REWARDS as usize] -= amount + port.tax;
            self.rewardedtotal += amount;

            // update port
            port.paid += amount;
            self.ports.insert(portnumber, &port);

            // emit Reward event
            self.env().emit_event(Reward {
                to: Some(address),
                amount: amount,
            });

            Ok(())
        }

        /// . get socket info
        #[ink(message)]
        pub fn socket(
            &self,
            contract: AccountId,
        ) -> Socket {
            
            match self.sockets.get(contract) {
                Some(socket) => socket,
                None => Default::default(),
            }
        }

        /// . create a new port that rewards contract can register with
        /// . eaech port tracks amount rewarded, tax collected, and if it is locked or not
        /// . a locked port may only be registered by the interlock network foundation
        #[ink(message)]
        #[openbrush::modifiers(only_owner)]
        pub fn create_port(
            &mut self,
            codehash: Hash,
            tax: Balance,
            cap: Balance,
            locked: bool,
            number: u16,
        ) -> PSP22Result<()> {

            // do we need an overflow check?
            // (ie, is it even possible to pass a u32, etc, port number?)

            let port = Port {
                hash: codehash,
                tax: tax,
                cap: cap,
                locked: locked,
                paid: 0,
                collected: 0,
            };

            self.ports.insert(number, &port);

            Ok(())
        }

        /// . get port info
        #[ink(message)]
        pub fn port(
            &self,
            port: u16,
        ) -> Port {
            
            match self.ports.get(port) {
                Some(port) => port,
                None => Default::default(),
            }
        }
    }

////////////////////////////////////////////////////////////////////////////
//// tests /////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
//
// INCOMPLETE
//
// . To view debug prints and assertion failures run test via:
//   cargo nightly+ test -- --nocapture
// . To view debug for specific method run test via:
//   cargo nightly+ test <test_function_here> -- --nocapture

    #[cfg(test)]
    mod tests {

        use super::*;
        use ink_lang as ink;
        use ink_lang::codegen::Env;

        /// . test if the default constructor does its job
        #[ink::test]
        fn constructor_works() {

            let ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // the rest
            assert_eq!(ILOCKtokenPSP22.monthspassed, 0);
            assert_eq!(ILOCKtokenPSP22.nextpayout, ILOCKtokenPSP22.env().block_timestamp() as u128 + ONE_MONTH);
        }

        /// . test if name getter does its job
        #[ink::test]
        fn name_works() {

            let ILOCKtokenPSP22 = ILOCKtoken::new_token();
            assert_eq!(ILOCKtokenPSP22.metadata.name, Some("Interlock Network".to_string()));
        }

        /// . test if symbol getter does its job
        #[ink::test]
        fn symbol_works() {

            let ILOCKtokenPSP22 = ILOCKtoken::new_token();
            assert_eq!(ILOCKtokenPSP22.metadata.symbol, Some("ILOCK".to_string()));
        }
        
        /// . test if decimals getter does its job
        #[ink::test]
        fn decimals_works() {

            let ILOCKtokenPSP22 = ILOCKtoken::new_token();
            assert_eq!(ILOCKtokenPSP22.metadata.decimals, 18);
        }

        /// . test if balance getter does its job
        #[ink::test]
        fn balance_of_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // charge alice's account
            ILOCKtokenPSP22.psp22.balances.insert(&accounts.alice, &100);

            assert_eq!(ILOCKtokenPSP22.balance_of(accounts.alice), 100);
        }

        /// . test if allowance getter does its job
        #[ink::test]
        fn allowance_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // Alice has not yet approved Bob
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 0);

            // Alice approves Bob for tokens
            assert_eq!(ILOCKtokenPSP22.approve(accounts.bob, 10), Ok(()));

            // Bob's new allowance reflects this approval
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 10);
        }

// Skipped: openbrush does checks that do cross-contract calls, sort of
//        /// . test if the transfer doer does its job
//        #[ink::test]
//        fn transfer_works() {
//
//            // construct contract and initialize accounts
//            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
//            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
//
//            // charge alice's account
//            ILOCKtokenPSP22.psp22.balances.insert(&accounts.alice, &100);
//
//            // alice transfers tokens to bob
//            assert_eq!(ILOCKtokenPSP22.transfer(accounts.bob, 10, Default::default()), Ok(()));
//
//            // Alice balance reflects transfer
//            assert_eq!(ILOCKtokenPSP22.balance_of(accounts.alice), 90);
//
//            // Bob balance reflects transfer
//            assert_eq!(ILOCKtokenPSP22.balance_of(accounts.bob), 10);
//
//            // Alice attempts transfer too large
//            assert_eq!(ILOCKtokenPSP22.transfer(accounts.bob, 100, Default::default()), Err(PSP22Error::InsufficientBalance));
//
//            // check all events that happened during the previous calls
//            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
//            assert_eq!(emitted_events.len(), 3);
//
//            // check the transfer event relating to the actual trasfer
//            assert_transfer_event(
//                &emitted_events[2],
//                Some(AccountId::from([0x01; ID_LENGTH])),
//                Some(AccountId::from([0x02; ID_LENGTH])),
//                10,
//            );
//        }

// Skipped: openbrush does checks that do cross-contract calls, sort of
//        /// . test if the approve does does its job
//        #[ink::test]
//        fn approve_works() {
//
//            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
//            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
//
//            // Alice approves bob to spend tokens
//            assert_eq!(ILOCKtokenPSP22.approve(accounts.bob, 10), Ok(()));
//
//            // Bob is approved to spend tokens owned by Alice
//            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 10);
//
//            // check all events that happened during previous calls
//            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
//            assert_eq!(emitted_events.len(), 3);
//
//            // check the approval event relating to the actual approval
//            assert_approval_event(
//                &emitted_events[2],
//                Some(AccountId::from([0x01; ID_LENGTH])),
//                Some(AccountId::from([0x02; ID_LENGTH])),
//                10,
//            );
//        }

// Skipped: openbrush does checks that do cross-contract calls, sort of
//        /// . test if the transfer-from doer does its job
//        #[ink::test]
//        fn transfer_from_works() {
//
//            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
//            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
//
//            // charge alice's account
//            ILOCKtokenPSP22.psp22.balances.insert(&accounts.alice, &100);
//
//            // Alice approves Bob for token transfers on her behalf
//            assert_eq!(ILOCKtokenPSP22.approve(accounts.bob, 10), Ok(()));
//
//            // set the contract owner as callee and Bob as caller
//            let contract = ink_env::account_id::<ink_env::DefaultEnvironment>();
//            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(contract);
//            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
//
//            // Check Bob's allowance
//            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 10);
//
//            // and Bob is caller now
//            assert_eq!(ILOCKtokenPSP22.env().caller(), accounts.bob);
//
//            // Bob transfers tokens from Alice to Eve
//            assert_eq!(ILOCKtokenPSP22.transfer_from(accounts.alice, accounts.eve, 10, Default::default()), Ok(()));
//
//            // Eve received the tokens
//            assert_eq!(ILOCKtokenPSP22.balance_of(accounts.eve), 10);
//
//            // Bob attempts a transferfrom too large
//            assert_eq!(ILOCKtokenPSP22.transfer_from(accounts.alice, accounts.eve, 100, Default::default()),
//                        Err(PSP22Error::InsufficientAllowance));
//
//            // check all events that happened during the previous callsd
//            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
//            assert_eq!(emitted_events.len(), 5);
//
//            // check that Transfer event was emitted        
//            assert_transfer_event(
//                &emitted_events[4],
//                Some(AccountId::from([0x01; ID_LENGTH])),
//                Some(AccountId::from([0x05; ID_LENGTH])),
//                10,
//            );
//        }

        /// . test if increase allowance does does its job
        #[ink::test]
        fn increase_allowance_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // Alice approves bob to spend tokens
            assert_eq!(ILOCKtokenPSP22.approve(accounts.bob, 10), Ok(()));

            // Bob is approved to spend tokens owned by Alice
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 10);

            // Alice increases Bobs allowance
            assert_eq!(ILOCKtokenPSP22.increase_allowance(accounts.bob, 10), Ok(()));

            // Bob is approved to spend extra tokens owned by Alice
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 20);
        }

        /// . test if decrease allowance does does its job
        #[ink::test]
        fn decrease_allowance_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // Alice approves bob to spend tokens
            assert_eq!(ILOCKtokenPSP22.approve(accounts.bob, 10), Ok(()));

            // Bob is approved to spend tokens owned by Alice
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 10);

            // Alice increases Bobs allowance
            assert_eq!(ILOCKtokenPSP22.decrease_allowance(accounts.bob, 5), Ok(()));

            // Bob is approved to spend extra tokens owned by Alice
            assert_eq!(ILOCKtokenPSP22.allowance(accounts.alice, accounts.bob), 5);
        }

        /// . test if wallet registration function works as intended 
        #[ink::test]
        fn register_stakeholder_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // bob's stakeholder data
            let share: Balance = 1_000_000;
            let pool: u8 = 3;

            // call registration function
            ILOCKtokenPSP22.register_stakeholder(accounts.bob, share, pool).unwrap();

            // verify registration stuck
            let this_stakeholder = ILOCKtokenPSP22.stakeholderdata.get(accounts.bob).unwrap();
            assert_eq!(this_stakeholder.paid, 0);
            assert_eq!(this_stakeholder.share, share);
            assert_eq!(this_stakeholder.pool, pool);
        }
     
// Skipped: openbrush does checks that do cross-contract calls, sort of
//        /// . test if the approve does does its job
//        #[ink::test]
//        fn distribute_tokens_works() {
//
//            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
//            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
//
//            // bob's stakeholder data
//            let share: Balance = 1_000_000;
//
//            let mut pool = 4;
//
//            // register bob, 6 month cliff, 36 vests (pool 4)
//            ILOCKtokenPSP22.register_stakeholder(accounts.bob, share, pool).unwrap();
//
//            // debug println header (if no capture is on)
//            ink_env::debug_println!("POOL 4, 36 MONTH VESTING PERIOD, 6 MONTH CLIFF");
//            // run distribution over 44 months (6 + 36 + 2)
//            for _month in 0..44 {
//
//                // get bob his monthly tokens
//                ILOCKtokenPSP22.distribute_tokens(accounts.bob).ok();
//
//                // print everything and check balances at each iteration
//                let this_stakeholder: StakeholderData = ILOCKtokenPSP22.stakeholderdata.get(accounts.bob).unwrap();
//                ink_env::debug_println!("month: {:?}\tpaid: {:?}", ILOCKtokenPSP22.monthspassed, this_stakeholder.paid);
//                assert_eq!(ILOCKtokenPSP22.balance_of(accounts.bob), this_stakeholder.paid);
//
//                // make time go on
//                ILOCKtokenPSP22.TESTING_increment_month();
//            }
//
//            // reset time
//            ILOCKtokenPSP22.monthspassed = 0;
//
//            pool = 1;
//
//            // register bob, 1 month cliff, 18 vests (pool 1)
//            ILOCKtokenPSP22.register_stakeholder(accounts.bob, share, pool).unwrap();
//            ILOCKtokenPSP22.psp22.balances.insert(&accounts.bob, &0);
//
//            // debug println header (if no capture is on)
//            ink_env::debug_println!("POOL 1, 18 MONTH VESTING PERIOD, 1 MONTH CLIFF");
//            // run distribution over 44 months (1 + 18 + 2)
//            for _month in 0..21 {
//
//                // get bob his monthly tokens
//                //ILOCKtokenPSP22.distribute_tokens(accounts.bob).ok();
//
//                // print everything and check balances at each iteration
//                let this_stakeholder: StakeholderData = ILOCKtokenPSP22.stakeholderdata.get(accounts.bob).unwrap();
//                ink_env::debug_println!("month: {:?}\tpaid: {:?}", ILOCKtokenPSP22.monthspassed, this_stakeholder.paid);
//                assert_eq!(ILOCKtokenPSP22.balance_of(accounts.bob), this_stakeholder.paid);
//
//                // make time go on
//                ILOCKtokenPSP22.TESTING_increment_month();
//            }
//
//            // reset time
//            ILOCKtokenPSP22.monthspassed = 0;
//
//            pool = 10;
//
//            // register bob, 0 month cliff, 48 vests (pool 10)
//            ILOCKtokenPSP22.register_stakeholder(accounts.bob, share, pool).unwrap();
//            ILOCKtokenPSP22.psp22.balances.insert(&accounts.bob, &0);
//
//            // debug println header (if no capture is on)
//            ink_env::debug_println!("POOL 10, 48 MONTH VESTING PERIOD, 0 MONTH CLIFF");
//            // run distribution over 44 months (0 + 48 + 2)
//            for _month in 0..50 {
//
//                // get bob his monthly tokens
//                ILOCKtokenPSP22.distribute_tokens(accounts.bob).ok();
//
//                // print everything and check balances at each iteration
//                let this_stakeholder: StakeholderData = ILOCKtokenPSP22.stakeholderdata.get(accounts.bob).unwrap();
//                ink_env::debug_println!("month: {:?}\tpaid: {:?}", ILOCKtokenPSP22.monthspassed, this_stakeholder.paid);
//                assert_eq!(ILOCKtokenPSP22.balance_of(accounts.bob), this_stakeholder.paid);
//
//                // make time go on
//                ILOCKtokenPSP22.TESTING_increment_month();
//            }
//        }

        /// . test if pool data getter does its job
        #[ink::test]
        fn pool_data_works() {

            let ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let pool = &POOLS[1];
            assert_eq!(ILOCKtokenPSP22.pool_data(1), (pool.name.to_string(),
                                                      pool.tokens,
                                                      pool.vests,
                                                      pool.cliffs));
        }

        /// . test if months passed getter does its job
        #[ink::test]
        fn months_passed_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            ILOCKtokenPSP22.monthspassed = 99;
            assert_eq!(ILOCKtokenPSP22.months_passed(), 99);
        }

        /// . test if burn does its job
        #[ink::test]
        fn burn_works() {

            let mut ILOCKtokenPSP22 = ILOCKtoken::new_token();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // charge alice's account
            ILOCKtokenPSP22.psp22.balances.insert(&accounts.alice, &100);

            // alice has her tokens burned by contract owner (herself in this case)
            ILOCKtokenPSP22.burn(accounts.alice, 100).unwrap();

            assert_eq!(ILOCKtokenPSP22.balance_of(accounts.alice), 0);
            assert_eq!(ILOCKtokenPSP22.total_supply(), 65_000_000 * DECIMALS_POWER10 - 100);
        }
    }
}
