#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{memory::format, storage};
use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    event Register {
        admin: AccountId,
        permission: bool,
    }

    /// This simple dummy contract has a `bool` value that can
    /// alter between `true` and `false` using the `flip` message.
    /// Users can retrieve its current state using the `get` message.
    struct PermissionedFlipper {
        /// The current state of our flag.
        value: storage::Value<bool>,
        owner: storage::Value<AccountId>,
        admins: storage::HashMap<AccountId, bool>
    }

    impl Deploy for PermissionedFlipper {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false);
            self.owner.set(env.caller());
        }
    }

    impl PermissionedFlipper {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) -> bool {
            let caller = env.caller();
            let is_owner = caller == *self.owner.get();
            let is_permissioned_admin = self.admins.get(&caller).map(|x|*x).unwrap_or(false);
            if is_owner || is_permissioned_admin {
                *self.value = !*self.value;
                env.println(&format!("Storage Value is flipped from {:?} to {:?} by caller: {:?}", !*self.value, *self.value, caller));
                true
            } else {
                env.println(&format!("Only the admin and owner of this smart contract can flip!"));
                false
            }
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            env.println(&format!("Storage Value: {:?}", *self.value));
            *self.value
        }

        /// Return the owner of this smart contract.
        pub(external) fn owner(&self) -> AccountId {
            let owner = *self.owner.get();
            env.println(&format!("Current owner: {:?}", owner));
            owner
        }

        /// Return if it's an admin of this smart contract.
        pub(external) fn is_admin(&self, who: AccountId) -> (bool, Option<bool>) {
            if let Some(permission) = self.admins.get(&who) {
                env.println(&format!("Admin: {:?}, permission {:?}", who, permission));
                (true, Some(*permission))
            } else {
                env.println(&format!("Account {:?} is not an admin", who));
                (false, None)
            }
        }

        /// Set the new permission of some admin.
        pub(external) fn set_permission(&mut self, who: AccountId, new_permission: bool) {
            let caller = env.caller();
            if self.owner != caller {
                env.println(&format!("Only the owner of this smart contract can set permission, current owner: {:?}, current caller: {:?}", *self.owner, caller));
                return;
            }
            if self.admins.get(&who).is_none() {
                env.println(&format!("Please register the admin: {:?} first.", who));
                return;
            }
            self.admins.insert(who, new_permission);
        }

        /// Register a new admin with the initial permission by the contract owner.
        pub(external) fn register(&mut self, who: AccountId, initial_permission: bool) {
            if self.owner != env.caller() {
                env.println(&format!("Only the owner of this smart contract can set permission, current owner: {:?}, caller: {:?}", *self.owner, env.caller()));
                return;
            }
            if self.admins.get(&who).is_some() {
                env.println(&format!("The admin: {:?} already exists", who));
                return;
            }
            self.admins.insert(who, initial_permission);
            env.println(&format!("The new admin: {:?}, permission: {:?}", who, self.admins.get(&who)));
            env.emit(Register {admin: who, permission: initial_permission});
        }

        /// Remove an existing admin by the contract owner.
        pub(external) fn remove(&mut self, admin: AccountId) {
            if self.owner != env.caller() {
                env.println(&format!("Only the owner of this smart contract has the permission to remove admin, current owner: {:?}", *self.owner));
                return;
            }
            if self.admins.get(&admin).is_none() {
                env.println(&format!("The admin: {:?} does not exist", admin));
                return;
            }
            self.admins.remove(&admin);
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    use std::convert::TryFrom;

    #[test]
    fn register_should_work() {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();
        let charlie = AccountId::try_from([0x2; 32]).unwrap();

        env::test::set_caller::<ink_core::env::DefaultSrmlTypes>(alice);

        let mut contract = PermissionedFlipper::deploy_mock();
        contract.register(bob, true);
        assert_eq!(contract.is_admin(bob), (true, Some(true)));

        contract.set_permission(bob, false);
        assert_eq!(contract.is_admin(bob), (true, Some(false)));

        env::test::set_caller::<ink_core::env::DefaultSrmlTypes>(charlie);
        assert_eq!(contract.flip(), false);

        env::test::set_caller::<ink_core::env::DefaultSrmlTypes>(alice);
        contract.register(charlie, true);

        env::test::set_caller::<ink_core::env::DefaultSrmlTypes>(charlie);
        assert_eq!(contract.get(), false);
        assert_eq!(contract.flip(), true);
        assert_eq!(contract.get(), true);
    }
}
