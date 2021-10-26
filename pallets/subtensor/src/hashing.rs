// use super::*;
// use sp_std::if_std; // Import into scope the if_std! macro.
// use sp_std::convert::TryInto;
// use sp_core::{H256, U256};
// use sp_io::hashing::sha2_256;
// // use sha3::{Digest, Sha3_256};

// impl<T: Config> Pallet<T> {

//     pub fn get_current_block_as_u64( ) -> u64 {
//         let block_as_u64: u64 = TryInto::try_into( system::Pallet::<T>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
//         block_as_u64
//     }

//     pub fn vec_to_hash( vec_hash: Vec<u8> ) -> H256 {
//         let de_ref_hash = &vec_hash; // b: &Vec<u8>
//         let de_de_ref_hash: &[u8] = &de_ref_hash; // c: &[u8]
//         let real_hash: H256 = H256::from_slice( de_de_ref_hash );
//         if_std! {
//             println!("real_hash: {:?}, vec_hash{:?}", real_hash, vec_hash);
//         }
//         return real_hash
//     }

//     /// Determine whether the given hash satisfies the given difficulty.
//     /// The test is done by multiplying the two together. If the product
//     /// overflows the bounds of U256, then the product (and thus the hash)
//     /// was too high.
//     pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
//         let num_hash = U256::from(&hash[..]);
//         let (value, overflowed) = num_hash.overflowing_mul(difficulty);
//         if_std! {
//             println!("num_hash: {:?}, value: {:?} overflowed: {:?}", num_hash, value, overflowed);
//         }
//         !overflowed
//     }

//     pub fn get_block_hash_from_u64 ( block_number: u64 ) -> H256 {
//         let block_number: T::BlockNumber = TryInto::<T::BlockNumber>::try_into( block_number ).ok().expect("convert u64 to block number.");
//         let block_hash_at_number: <T as frame_system::Config>::Hash = system::Pallet::<T>::block_hash( block_number );
//         let vec_hash: Vec<u8> = block_hash_at_number.as_ref().into_iter().cloned().collect();
//         let deref_vec_hash: &[u8] = &vec_hash; // c: &[u8]
//         let real_hash: H256 = H256::from_slice( deref_vec_hash );
//         if_std! {
//             println!("block_number: {:?}, vec_hash: {:?}, real_hash: {:?}", block_number, vec_hash, real_hash);
//         }
//         return real_hash;
//     }

//     pub fn create_seal_hash( block_hash: H256, nonce: U256 ) -> H256 {
//         // Do a concat of the block_hash + nonce.
//         let hash_as_bytes: &[u8] = block_hash.as_bytes();
//         let nonce_bytes: &[u8; 32] = &[
//             nonce.byte(0), nonce.byte(1), nonce.byte(2), nonce.byte(3), 
//             nonce.byte(4), nonce.byte(5), nonce.byte(6), nonce.byte(7), 
//             nonce.byte(8), nonce.byte(9), nonce.byte(10), nonce.byte(11), 
//             nonce.byte(12), nonce.byte(13), nonce.byte(14), nonce.byte(15), 
//             nonce.byte(16), nonce.byte(17), nonce.byte(18), nonce.byte(19), 
//             nonce.byte(20), nonce.byte(21), nonce.byte(22), nonce.byte(23), 
//             nonce.byte(24),nonce.byte(25), nonce.byte(26), nonce.byte(27), 
//             nonce.byte(28), nonce.byte(29) ,nonce.byte(30), nonce.byte(31), 
//         ];
//         let seal: Vec<u8> = [hash_as_bytes, nonce_bytes].concat();

//         // Use sha256 to create the hash.
//         let seal_hash: [u8; 32] = sha2_256( &seal );
//         let seal_hash: H256 = H256::from_slice( &seal_hash );
//         if_std! {
//             println!("block_hash: {:?}, nonce: {:?}, seal: {:?}, seal_hash: {:?}", block_hash, nonce, seal, seal_hash);
//         }
//         return seal_hash;
//     }


//     pub fn check_work ( block_number: u64, block_hash: H256, nonce: U256, difficulty: U256, work: H256 ) -> bool {

//         // Check block number range.
//         let current_block_number: u64 = Self::get_current_block_as_u64();
//         if current_block_number < block_number {
//             return false
//         }

//         // Check that the submitted block hash is the same as the block hash at this height.
//         let block_hash_at_number: H256 = Self::get_block_hash_from_u64( block_number );
//         if block_hash_at_number != block_hash {
//             return false;
//         }

//         // Check that the difficulty has been met by the submitted work.
//         if !Self::hash_meets_difficulty( &work,  difficulty ) {
//             return false;
//         }

//         // Check that the seal matches the work.
//         let seal: H256 = Self::create_seal_hash( block_hash, nonce );
//         if seal != work {
//             return false;
//         }
//         return true;
//     }

// }