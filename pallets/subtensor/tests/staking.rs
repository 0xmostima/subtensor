use frame_support::{assert_ok};
use frame_system::{Config};
mod mock;
use mock::*;
use mock::{TestXt};
use frame_support::sp_runtime::DispatchError;
use pallet_subtensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};

/***********************************************************
	staking::add_stake() tests
************************************************************/


#[test]
fn test_add_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let account_id = 0;
		let stake = 5000;

        let call = Call::Subtensor(SubtensorCall::add_stake(account_id, stake));

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes
		});
	});
}

/************************************************************
	This test also covers any signed extensions
************************************************************/

#[test]
fn test_add_stake_transaction_fee_ends_up_in_transaction_fee_pool() {
	let test_neuron_cold_key = 1;
	let test_neuron_hot_key = 2;

	// Give account id 1 10^9 rao ( 1 Tao )
	let balances = vec![(test_neuron_cold_key, 1_000_000_000)];

	mock::test_ext_with_balances(balances).execute_with(|| {
		// Register neuron_1
		let test_neuron = subscribe_ok_neuron(test_neuron_hot_key, test_neuron_cold_key);

		// Verify start situation
        let start_balance = Subtensor::get_coldkey_balance(&test_neuron_cold_key);
		let start_stake = Subtensor::get_stake_of_neuron_hotkey_account_by_uid(test_neuron.uid);
		assert_eq!(start_balance, 1_000_000_000);
		assert_eq!(start_stake, 0);

		let call = Call::Subtensor(SubtensorCall::add_stake(test_neuron_hot_key, 500_000_000));
		let xt = TestXt::new(call, mock::sign_extra(test_neuron_cold_key, 0));
		let result = mock::Executive::apply_extrinsic( xt );
		assert_ok!(result);

		let end_balance = Subtensor::get_coldkey_balance(&test_neuron_cold_key);
		assert_eq!(end_balance, 500_000_000);
	});
}


#[test]
fn test_add_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 533453;
		let ip = ipv4(8,8,8,8);
		let port = 66;
		let ip_type = 4;
		let modality = 0;
		let coldkey_account_id = 55453;

		// Subscribe neuron
		let neuron = subscribe_neuron(hotkey_account_id, ip,port, ip_type, modality,  coldkey_account_id);

		// Give it some $$$ in his coldkey balance
		Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

		// Check we have zero staked before transfer
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

		// Also total stake should be zero
		assert_eq!(Subtensor::get_total_stake(), 0);

		// Transfer to hotkey account, and check if the result is ok
		assert_ok!(Subtensor::add_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, 10000));

		// Check if stake has increased
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 10000);

		// Check if balance has  decreased
		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);

		// Check if total stake has increased accordingly.
		assert_eq!(Subtensor::get_total_stake(), 10000);
	});
}

#[test]
fn test_dividends_with_run_to_block() {
	new_test_ext().execute_with(|| {
       	let neuron_src_hotkey_id = 1;
		let neuron_dest_hotkey_id = 2;
		let coldkey_account_id = 667;

		let initial_stake:u64 = 5000;

		// Subscribe neuron, this will set a self weight
		let _adam = subscribe_ok_neuron(0, coldkey_account_id);
		let neuron_src = subscribe_ok_neuron(neuron_src_hotkey_id, coldkey_account_id);
		let neuron_dest = subscribe_ok_neuron(neuron_dest_hotkey_id, coldkey_account_id);

		// Add some stake to the hotkey account, so we can test for emission before the transfer takes place
		Subtensor::add_stake_to_neuron_hotkey_account(neuron_src.uid, initial_stake);

		// Check if the initial stake has arrived
		assert_eq!( Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_src.uid), initial_stake );

		assert_eq!( Subtensor::get_neuron_count(), 3 );

		// Run a couple of blocks to check if emission works
		run_to_block( 2 );

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_src.uid), initial_stake);

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_dest.uid), 0);
	});
}

#[test]
fn test_add_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 654; // bogus
		let amount = 20000 ; // Not used

		let result = Subtensor::add_stake(<<Test as Config>::Origin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_add_stake_err_not_active() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 435445; // Not active id
		let hotkey_account_id = 54544;
		let amount = 1337;

		let result = Subtensor::add_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}


#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;

		let _neuron = subscribe_neuron(hotkey_id, ipv4(8, 8, 8, 8), 66, 4, 0, coldkey_id);

		// Perform the request which is signed by a different cold key
		let result = Subtensor::add_stake(<<Test as Config>::Origin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_add_stake_err_not_enough_belance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;


		let _neuron = subscribe_neuron(hotkey_id, ipv4(8, 8, 8, 8), 66, 4, 0, coldkey_id);

		// Lets try to stake with 0 balance in cold key account
		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_id), 0);
		let result = Subtensor::add_stake(<<Test as Config>::Origin>::signed(coldkey_id), hotkey_id, 60000);

		assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
	});
}

/***********************************************************
	staking::remove_stake() tests
************************************************************/

#[test]
fn test_remove_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
        let account_id = 0;
		let stake = 5000;

		let call = Call::Subtensor(SubtensorCall::remove_stake(account_id, stake));

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes
		});
	});
}

// #[test]
// fn test_remove_stake_ok_transaction_fee_ends_up_in_transaction_fee_pool() {
// 	let coldkey_id = 667;
// 	let initial_stake : u64 = 1_000_000_000;
// 	let hotkey_id = 1;

// 	test_ext_with_balances(vec![(coldkey_id, initial_stake as u128)]).execute_with(|| {
//         let _adam = subscribe_ok_neuron(0,667);
// 		let _neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);
// 		assert_ok!(Subtensor::add_stake(Origin::signed(coldkey_id), hotkey_id, initial_stake));


// 		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(hotkey_id), 1_000_000_000);

// 		let call = Call::Subtensor(SubtensorCall::remove_stake(hotkey_id, 500_000_000));
// 		let xt = TestXt::new(call, mock::sign_extra(coldkey_id, 0));
// 		let result = mock::Executive::apply_extrinsic(xt);
// 		assert_ok!(result);

// 		assert!(Subtensor::get_transaction_fee_pool() > 0);
// 	});
// }


#[test]
fn test_remove_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 4343;
		let hotkey_account_id = 4968585;
		let amount = 10000;

		// Let's spin up a neuron
		let neuron = subscribe_neuron(hotkey_account_id, ipv4(8,8,8,8), 66, 4, 0, coldkey_account_id);

		// Some basic assertions
		assert_eq!(Subtensor::get_total_stake(), 0);
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);
		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);

		// Give the neuron some stake to remove
		Subtensor::add_stake_to_neuron_hotkey_account(neuron.uid, amount);

		// Do the magic
		assert_ok!(Subtensor::remove_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), amount as u128);
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);
	});
}

// #[test]
// fn test_remove_stake_ok_with_emission() {
// 	new_test_ext().execute_with(|| {
//         let coldkey_account_id = 4343;
// 		let hotkey_neuron_src: u64 = 4968585;
// 		let hotkey_neuron_dest: u64 = 78979;
// 		let amount = 10000; // Amount to be removed
// 		let initial_amount = 20000; // This will be added before the function UT is called, to trigger an emit

// 		// Add neurons
// 		let _adam = subscribe_ok_neuron(0, coldkey_account_id);
// 		let neuron_src = subscribe_ok_neuron(hotkey_neuron_src, coldkey_account_id);
// 		let neuron_dest = subscribe_ok_neuron(hotkey_neuron_dest, coldkey_account_id);

// 		// Set neuron_src weight to neuron_dest
// 		let _ = Subtensor::set_weights(Origin::signed(hotkey_neuron_src), vec![neuron_dest.uid], vec![100]);

// 		// Add the stake to the hotkey account
// 		Subtensor::add_stake_to_neuron_hotkey_account(neuron_src.uid, initial_amount);

// 		// Some basic assertions
// 		assert_eq!(Subtensor::get_total_stake(), initial_amount);
// 		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_src.uid), initial_amount);
// 		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);

// 		// Run a couple of blocks
// 		run_to_block(100);

// 		// Perform the remove_stake operation
// 		assert_ok!(Subtensor::remove_stake(Origin::signed(coldkey_account_id), hotkey_neuron_src, amount));

// 		// The amount of stake left should be the same as the inital amount and the removed amount
// 		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_src.uid), initial_amount - amount);

// 		// The total stake should be bigger than the initial amount - amount
// 		assert!(Subtensor::get_total_stake() > initial_amount - amount);

// 		// The coldkey balance should be equal to the removed stake
// 		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), amount as u128);

// 		// The stake of neuron_dest should be > 0, due to emission to this neuron
// 		assert!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron_dest.uid) > 0);
// 	});
// }

#[test]
fn test_remove_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id : u64 = 4968585;
		let amount = 10000; // Amount to be removed

		let result = Subtensor::remove_stake(<<Test as Config>::Origin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}


#[test]
fn test_remove_stake_err_not_active() {
	new_test_ext().execute_with(|| {
        let coldkey_account_id = 435445;
		let hotkey_account_id = 54544; // Not active id
		let amount = 1337;

		let result = Subtensor::add_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}

#[test]
fn test_remove_stake_err_neuron_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;

		let _neuron = subscribe_ok_neuron(hotkey_id,coldkey_id);

		// Perform the request which is signed by a different cold key
		let result = Subtensor::remove_stake(<<Test as Config>::Origin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_remove_stake_no_enough_stake() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let amount = 10000;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);

		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

		let result = Subtensor::remove_stake(<<Test as Config>::Origin>::signed(coldkey_id), hotkey_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
	});
}


/***********************************************************
	staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
		let result = Subtensor::get_coldkey_balance(&coldkey_account_id);

		// Arbitrary account should have 0 balance
		assert_eq!(result, 0);

	});
}


#[test]
fn test_get_coldkey_balance_with_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
		let amount = 1337;

		// Put the balance on the account
		Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, amount);

		let result = Subtensor::get_coldkey_balance(&coldkey_account_id);

		// Arbitrary account should have 0 balance
		assert_eq!(result, amount);

	});
}


/***********************************************************
	staking::add_stake_to_neuron_hotkey_account() tests
************************************************************/
#[test]
fn test_add_stake_to_neuron_hotkey_account_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);

		// There is not stake in the system at first, so result should be 0;
		assert_eq!(Subtensor::get_total_stake(), 0);

		// Gogogo
		Subtensor::add_stake_to_neuron_hotkey_account(neuron.uid, amount);

		// The stake that is now in the account, should equal the amount
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), amount);

		// The total stake should have been increased by the amount -> 0 + amount = amount
		assert_eq!(Subtensor::get_total_stake(), amount);
	});
}

/************************************************************
	staking::remove_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_stake_from_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);

		// Add some stake that can be removed
		Subtensor::add_stake_to_neuron_hotkey_account(neuron.uid, amount);

		// Prelimiary checks
		assert_eq!(Subtensor::get_total_stake(), amount);
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), amount);

		// Remove stake
		Subtensor::remove_stake_from_neuron_hotkey_account(neuron.uid, amount);

		// The stake on the hotkey account should be 0
		assert_eq!(Subtensor::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

		// The total amount of stake should be 0
		assert_eq!(Subtensor::get_total_stake(), 0);
	});
}


/************************************************************
	staking::increase_total_stake() tests
************************************************************/
#[test]
fn test_increase_total_stake_ok() {
	new_test_ext().execute_with(|| {
		let increment = 10000;

        assert_eq!(Subtensor::get_total_stake(), 0);
		Subtensor::increase_total_stake(increment);
		assert_eq!(Subtensor::get_total_stake(), increment);
	});
}

#[test]
#[should_panic]
fn test_increase_total_stake_panic_overflow() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = u64::MAX;
		let increment : u64 = 1;

		// Setup initial total stake
		Subtensor::increase_total_stake(initial_total_stake);
		Subtensor::increase_total_stake(increment); // Should trigger panic
	});
}

/************************************************************
	staking::decrease_total_stake() tests
************************************************************/
#[test]
fn test_decrease_total_stake_ok() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = 10000;
		let decrement = 5000;

		Subtensor::increase_total_stake(initial_total_stake);
		Subtensor::decrease_total_stake(decrement);

		// The total stake remaining should be the difference between the initial stake and the decrement
		assert_eq!(Subtensor::get_total_stake(), initial_total_stake - decrement);
	});
}

#[test]
#[should_panic]
fn test_decrease_total_stake_panic_underflow() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = 10000;
		let decrement = 20000;

		Subtensor::increase_total_stake(initial_total_stake);
		Subtensor::decrease_total_stake(decrement); // Should trigger panic
	});
}

/************************************************************
	staking::add_balance_to_coldkey_account() tests
************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 4444322;
		let amount = 50000;

		Subtensor::add_balance_to_coldkey_account(&coldkey_id, amount);
		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_id), amount);

	});
}

/***********************************************************
	staking::remove_balance_from_coldkey_account() tests
************************************************************/


#[test]
fn test_remove_balance_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Put some $$ on the bank
		Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), ammount);

		// Should be able to withdraw without hassle
		let result = Subtensor::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, true);
	});
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Try to remove stake from the coldkey account. This should fail,
		// as there is no balance, nor does the account exist
		let result = Subtensor::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, false);
	});
}

/************************************************************
	staking::neuron_belongs_to_coldkey() tests
************************************************************/
#[test]
fn test_neuron_belongs_to_coldkey_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4434334;
		let coldkey_id = 34333;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);
		assert_eq!(Subtensor::neuron_belongs_to_coldkey(&neuron, &coldkey_id), true);
	});
}

#[test]
fn test_neurong_belongs_to_coldkey_err() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4434334;
		let coldkey_id = 34333;
		let other_coldkey_id = 8979879;

		let neuron = subscribe_ok_neuron(hotkey_id, other_coldkey_id);
		assert_eq!(Subtensor::neuron_belongs_to_coldkey(&neuron, &coldkey_id), false);
	});
}

/************************************************************
	staking::can_remove_balance_from_coldkey_account() tests
************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 5000;

		Subtensor::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(Subtensor::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), true);
	});
}


#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 20000;

		Subtensor::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(Subtensor::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), false);
	});
}

/************************************************************
	staking::has_enough_stake() tests
************************************************************/
#[test]
fn test_has_enough_stake_yes() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 10000;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);

		Subtensor::add_stake_to_neuron_hotkey_account(neuron.uid, intial_amount);
		assert_eq!(Subtensor::has_enough_stake(&neuron, 5000), true);
	});
}

#[test]
fn test_has_enough_stake_no() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 0;

		let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);

		Subtensor::add_stake_to_neuron_hotkey_account(neuron.uid, intial_amount);
		assert_eq!(Subtensor::has_enough_stake(&neuron, 5000), false);

	});
}


/****************************************************************************
	staking::create_hotkey_account() and staking::has_hotkey_account() tests
*****************************************************************************/
#[test]
fn test_has_hotkey_account_no() {
	new_test_ext().execute_with(|| {
        assert_eq!(Subtensor::has_hotkey_account(&8888), false);
	});
}

#[test]
fn test_has_hotkey_account_yes() {
	new_test_ext().execute_with(|| {
        let uid = 666;

		Subtensor::create_hotkey_account(uid);
		assert_eq!(Subtensor::has_hotkey_account(&uid), true);
	});
}


/************************************************************
	staking::calculate_stake_fraction_for_neuron() tests
************************************************************/
#[test]
fn test_calculate_stake_fraction_for_neuron_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_ids: Vec<u64> = vec![545345, 809809809];
		let coldkey_ids: Vec<u64> = vec![98748974892, 8798798];
		let intial_stakes = vec![10000,10000];

		let neurons = vec![
			subscribe_ok_neuron(hotkey_ids[0], coldkey_ids[1]),
			subscribe_ok_neuron(hotkey_ids[1], coldkey_ids[1])
		];

		// Add stake to neurons
		Subtensor::add_stake_to_neuron_hotkey_account(neurons[0].uid, intial_stakes[0]);
		Subtensor::add_stake_to_neuron_hotkey_account(neurons[1].uid, intial_stakes[1]);

		// Total stake should now be 200000
		assert_eq!(Subtensor::get_total_stake(), 20000);

		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[0]), 0.5);
		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[1]), 0.5);
	});
}

#[test]
fn test_calculate_stake_fraction_for_neuron_no_total_stake() {
	new_test_ext().execute_with(|| {
		let hotkey_ids: Vec<u64> = vec![545345, 809809809];
		let coldkey_ids: Vec<u64> = vec![98748974892, 8798798];

		let neurons = vec![
			subscribe_ok_neuron(hotkey_ids[0], coldkey_ids[1]),
			subscribe_ok_neuron(hotkey_ids[1], coldkey_ids[1])
		];

		// Add NO stake to neurons

		// Total stake should now be 0
		assert_eq!(Subtensor::get_total_stake(), 0);

		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[0]), 0);
		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[1]), 0);

	});
}

#[test]
fn test_calculate_stake_fraction_for_neuron_no_neuron_stake() {
	new_test_ext().execute_with(|| {
	 	let hotkey_ids: Vec<u64> = vec![545345, 809809809];
		let coldkey_ids: Vec<u64> = vec![98748974892, 8798798];
		let intial_stakes = vec![10000,0];

		let neurons = vec![
			subscribe_ok_neuron(hotkey_ids[0], coldkey_ids[1]),
			subscribe_ok_neuron(hotkey_ids[1], coldkey_ids[1])
		];

		// Add stake to neurons
		Subtensor::add_stake_to_neuron_hotkey_account(neurons[0].uid, intial_stakes[0]);
		Subtensor::add_stake_to_neuron_hotkey_account(neurons[1].uid, intial_stakes[1]);

		// Total stake should now be 100000
		assert_eq!(Subtensor::get_total_stake(), 10000);

		// This guy should get the full proportion of the stake
		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[0]), 1);

		// This guy gets 0, because no stake
		assert_eq!(Subtensor::calculate_stake_fraction_for_neuron(&neurons[1]), 0);
	});
}