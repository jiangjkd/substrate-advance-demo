use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

/// 创建kitty 够足质押
#[test]
fn owned_kitties_can_append_values() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        // assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
        assert_ok!(Kitties::create(Origin::signed(1)));
        assert_eq!(
        System::events()[1].event,
        TestEvent::kitties_event(Event::<Test>::Created(1u64, 0)))
    })
}
//创建kitty 不够质押
#[test]
fn owner_kitties_failed_when_no_enought_money() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_noop!(Kitties::create(Origin::signed(6)), Error::<Test>::MoneyNotEnough);
    })
}

// 转移kitty 失败,Kitty 不存在
#[test]
fn transfer_kitty_failed_when_no_exists() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        // assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
        assert_ok!(Kitties::create(Origin::signed(1)));
        assert_noop!(
            Kitties::transfer(Origin::signed(1), 2, 10),
            Error::<Test>::InvalidKittyId
        );
    })
}

// 转移kitty 不拥有kitty测试
#[test]
fn transfer_kitties_no_owned() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Kitties::create(Origin::signed(1)));
        let _id = Kitties::kitties_count();
        assert_ok!(Kitties::transfer(Origin::signed(1), 2 , 999));
        assert_noop!(
            Kitties::transfer(Origin::signed(1), 2, 999),
            Error::<Test>::NotKittyOwner
        );
    })
}

// 转移kitty 拥有kitty测试 测试通过
#[test]
fn transfer_kitties() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Kitties::create(Origin::signed(1)));
        let id = Kitties::kitties_count();
        assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id - 1));
        assert_noop!(
            Kitties::transfer(Origin::signed(1), 2, id - 1),
            Error::<Test>::NotKittyOwner
        );
    })
}
// 测度相同父母  无法breed
#[test]
fn breed_kitty_fail_when_same() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let _ = Kitties::create(Origin::signed(1));

        assert_noop!(
            Kitties::breed(Origin::signed(1), 0, 0),
            Error::<Test>::RequireDifferentParent
        );
    })
}
// 不存在的kitty,无法参与breed
#[test]
fn breed_kitty_fail_when_not_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Kitties::breed(Origin::signed(1), 0, 1),
            Error::<Test>::InvalidKittyId
        );
    })
}

