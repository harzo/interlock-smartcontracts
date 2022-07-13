/****************************************************************
 * ILOCKsupreme Solana Contract      
 ****************************************************************/

#![allow(non_snake_case)]

use crate::utils::utils::*;

pub enum ContractInstruction {

    ProgramInit {

        bumpGLOBAL: u8,
        seedGLOBAL: Vec<u8>,
    },

    UpdateGlobal {

        updateFlags: u32,
        values: [u32; VALUES],
    },
    
    CreateUser {

        bumpUSER: u8,
        seedUSER: Vec<u8>,
    },

    FillAccount {

    },

    CreateStake {

        bumpSTAKE: u8,
        seedSTAKE: Vec<u8>,
        amount: u128,
        valence: u8,
    },

    SettleEntity {

        determination: u8,
    },
    
    CloseStake {

    },

    CreateEntity {

        bumpSTAKE: u8,
        seedSTAKE: Vec<u8>,
        bumpENTITY: u8,
        seedENTITY: Vec<u8>,
        amount: u128,
        valence: u8,
    },
}

