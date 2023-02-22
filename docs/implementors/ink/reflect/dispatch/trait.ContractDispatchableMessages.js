(function() {var implementors = {
"ilockmvp":[["impl <a class=\"trait\" href=\"ink/reflect/dispatch/trait.ContractDispatchableMessages.html\" title=\"trait ink::reflect::dispatch::ContractDispatchableMessages\">ContractDispatchableMessages</a>&lt;/// - ILOCKmvp struct contains overall storage data for contract\n    #[ink(storage)]\n    #[derive(Default, Storage)]\n    pub struct ILOCKmvp {\n\n        // ABSOLUTELY DO NOT CHANGE THE ORDER OF THESE VARIABLES\n        // OR TYPES IF UPGRADING THIS CONTRACT!!!\n\n        /// - Openbrush PSP22.\n        #[storage_field]\n        psp22: psp22::Data,\n\n        /// - Openbrush ownership extension.\n        #[storage_field]\n\t\townable: ownable::Data,\n\n        /// - Openbrush metadata extension.\n        #[storage_field]\n        metadata: metadata::Data,\n\n        /// - ILOCK Rewards info.\n        #[storage_field]\n        reward: RewardData,\n\n        /// - ILOCK token pool info.\n        #[storage_field]\n        pool: TokenPools,\n\n        /// - ILOCK vesting info.\n        #[storage_field]\n        vest: VestData,\n\n        /// - ILOCK connecting application contract info\n        #[storage_field]\n        app: AppData,\n    }&gt; for <a class=\"struct\" href=\"ilockmvp/ilockmvp/struct.ILOCKmvp.html\" title=\"struct ilockmvp::ilockmvp::ILOCKmvp\">ILOCKmvp</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()