use crate::history::ProfileChangeHistory;
use crate::ProfileIdentifier;

#[derive(Clone, Debug)]
pub struct Profile {
    profile_id: ProfileIdentifier,
    change_history: ProfileChangeHistory,
}
#[derive(Clone, Debug)]
pub struct RemoteProfile {
    profile_id: ProfileIdentifier,
}

pub trait ProfileCreator<T> {
    fn new(from: T) -> Profile;
}

impl ProfileCreator<&str> for Profile {
    fn new(s: &str) -> Profile {
        Profile {
            profile_id: ProfileIdentifier::from(s),
            change_history: ProfileChangeHistory::default(),
        }
    }
}

impl ProfileCreator<ProfileIdentifier> for Profile {
    fn new(profile_id: ProfileIdentifier) -> Profile {
        Profile {
            profile_id,
            change_history: ProfileChangeHistory::default(),
        }
    }
}

impl RemoteProfile {
    pub fn new(s: &str) -> Self {
        RemoteProfile {
            profile_id: ProfileIdentifier::from(s),
        }
    }
}

pub trait ProfileOps {
    fn profile_id(&self) -> ProfileIdentifier;
}

impl ProfileOps for Profile {
    fn profile_id(&self) -> ProfileIdentifier {
        self.profile_id.clone()
    }
}

impl ProfileOps for RemoteProfile {
    fn profile_id(&self) -> ProfileIdentifier {
        self.profile_id.clone()
    }
}

pub struct ProfileSet {
    profiles: Vec<Profile>,
}

pub trait ProfileSetCreator<T> {
    fn new(from: T) -> ProfileSet;
}

impl ProfileSetCreator<Profile> for ProfileSet {
    fn new(initial: Profile) -> ProfileSet {
        ProfileSet {
            profiles: vec![initial],
        }
    }
}

pub trait ProfileSetOps {
    fn get(&self, profile_id: ProfileIdentifier) -> Option<&Profile>;
}

impl ProfileSetOps for ProfileSet {
    fn get(&self, profile_id: ProfileIdentifier) -> Option<&Profile> {
        for profile in &self.profiles {
            if profile_id == profile.profile_id {
                return Some(profile);
            }
        }
        None
    }
}

#[test]
fn profile_test() {
    let pid = ProfileIdentifier::from("1234");
    let p = Profile::new(pid.clone());
    let s = ProfileSet::new(p);

    let p2 = s.get(pid.clone());
    assert!(p2.is_some());

    let p2 = p2.unwrap();
    assert_eq!(p2.profile_id, pid);
}
