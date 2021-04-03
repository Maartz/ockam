use crate::entity::profile::{
    Profile, ProfileCreator, ProfileOps, ProfileSet, ProfileSetCreator, ProfileSetOps,
    RemoteProfile,
};
use crate::ProfileIdentifier;
use ockam_vault::SoftwareVault;
use ockam_vault_core::CompositeVault;

pub type EntityVault = Box<dyn CompositeVault>;

pub struct SecureChannel; // TODO: place holder type

pub struct Entity {
    default_profile: ProfileIdentifier,
    profiles: ProfileSet,
    vault: EntityVault,
}

pub trait SecureChannelCreator {
    fn secure_channel_create<P: ProfileOps, R: ProfileOps>(
        local: &P,
        _remote: &R,
    ) -> crate::Result<SecureChannel> {
        println!(
            "secure_channel_create: {:?} -> {:?}",
            local.profile_id(),
            _remote.profile_id()
        );
        todo!()
    }
}

impl ProfileOps for Entity {
    fn profile_id(&self) -> ProfileIdentifier {
        self.get_default_profile().profile_id()
    }
}

impl SecureChannelCreator for Entity {
    fn secure_channel_create<P: ProfileOps, R: ProfileOps>(
        _local: &P,
        _remote: &R,
    ) -> crate::Result<SecureChannel> {
        todo!()
    }
}

impl Entity {
    fn new(vault: EntityVault) -> Entity {
        let default_profile = "keyid";
        let default = Profile::new(ProfileIdentifier::from(default_profile.clone()));
        let profiles = ProfileSet::new(default.clone());
        Entity {
            default_profile: default.profile_id(),
            profiles,
            vault,
        }
    }

    fn get_default_profile(&self) -> &Profile {
        if let Some(default) = self.profiles.get(self.default_profile.clone()) {
            default
        } else {
            panic!("Entity missing default profile")
        }
    }

    fn create_secure_channel(&self, remote: &RemoteProfile) -> crate::Result<SecureChannel> {
        Entity::secure_channel_create(self, remote)
    }
}

#[test]
fn entity_test() {
    let vault = Box::new(SoftwareVault::new());
    let alice = Entity::new(vault);

    let bob = RemoteProfile::new("01234");

    alice.create_secure_channel(&bob).unwrap();
}
