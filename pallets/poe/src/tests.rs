use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

// 创建存证 长度限制不超过5个字节 此测试通过
#[test]
fn create_claim_works() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1];
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        assert_eq!(Proofs::<Test>::get(&claim), (1, system::Module::<Test>::block_number()));
    })
}

// 撤销存证
#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
    })
}

// 转移存证
#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
    })
}

// 存证长度限制不超过5个字节 此测试失败
#[test]
fn create_claim_limit_length_works() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1, 2, 3, 4, 5];
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimExceedLength
        );
    })
}
//////////////////////////////////////////////////////////
// 其他
/*#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with( || {
        let claim: Vec<u8> = vec![0, 1];
        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    })
}*/