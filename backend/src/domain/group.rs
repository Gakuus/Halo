use super::{
    error::DomainError,
    value_objects::{GroupId, GroupName, Timestamp, UserId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
}

const MAX_GROUP_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct GroupMember {
    user_id: UserId,
    role: GroupRole,
    joined_at: Timestamp,
}

impl GroupMember {
    pub fn new(user_id: UserId, role: GroupRole) -> Self {
        Self {
            user_id,
            role,
            joined_at: Timestamp::now(),
        }
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn role(&self) -> GroupRole {
        self.role
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    id: GroupId,
    name: GroupName,
    owner_id: UserId,
    members: Vec<GroupMember>,
    created_at: Timestamp,
}

impl Group {
    pub fn new(
        name: GroupName,
        owner_id: UserId,
        initial_members: Vec<UserId>,
    ) -> Result<Self, DomainError> {
        if initial_members.is_empty() {
            return Err(DomainError::Internal(
                "group must have at least one member besides owner".into(),
            ));
        }

        let mut members = Vec::with_capacity(initial_members.len() + 1);

        members.push(GroupMember::new(owner_id, GroupRole::Owner));

        for member_id in initial_members {
            if member_id == owner_id {
                continue;
            }
            if members.iter().any(|m| m.user_id == member_id) {
                return Err(DomainError::MemberAlreadyExists);
            }
            members.push(GroupMember::new(member_id, GroupRole::Member));
        }

        Ok(Self {
            id: GroupId::new(),
            name,
            owner_id,
            members,
            created_at: Timestamp::now(),
        })
    }

    pub fn id(&self) -> &GroupId {
        &self.id
    }

    pub fn name(&self) -> &GroupName {
        &self.name
    }

    pub fn owner_id(&self) -> &UserId {
        &self.owner_id
    }

    pub fn members(&self) -> &[GroupMember] {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_member(&self, user_id: &UserId) -> bool {
        self.members.iter().any(|m| m.user_id == *user_id)
    }

    pub fn is_owner(&self, user_id: &UserId) -> bool {
        self.owner_id == *user_id
    }

    pub fn is_admin_or_owner(&self, user_id: &UserId) -> bool {
        self.members
            .iter()
            .any(|m| m.user_id == *user_id && (m.role == GroupRole::Owner || m.role == GroupRole::Admin))
    }

    pub fn add_member(&mut self, user_id: UserId) -> Result<(), DomainError> {
        if self.members.len() >= MAX_GROUP_SIZE {
            return Err(DomainError::GroupFull);
        }
        if self.is_member(&user_id) {
            return Err(DomainError::MemberAlreadyExists);
        }
        self.members
            .push(GroupMember::new(user_id, GroupRole::Member));
        Ok(())
    }

    pub fn remove_member(&mut self, user_id: &UserId) -> Result<(), DomainError> {
        if self.owner_id == *user_id {
            return Err(DomainError::Internal(
                "cannot remove the group owner".into(),
            ));
        }
        let pos = self
            .members
            .iter()
            .position(|m| m.user_id == *user_id)
            .ok_or(DomainError::NotMember)?;
        self.members.remove(pos);
        Ok(())
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group_success() {
        let name = GroupName::new("test-group").unwrap();
        let owner = UserId::new();
        let member = UserId::new();
        let group = Group::new(name, owner, vec![member]);
        assert!(group.is_ok());
        assert_eq!(group.unwrap().member_count(), 2);
    }

    #[test]
    fn test_create_group_no_members() {
        let name = GroupName::new("test-group").unwrap();
        let owner = UserId::new();
        let group = Group::new(name, owner, vec![]);
        assert!(group.is_err());
    }

    #[test]
    fn test_is_member() {
        let name = GroupName::new("test-group").unwrap();
        let owner = UserId::new();
        let member = UserId::new();
        let group = Group::new(name, owner, vec![member]).unwrap();
        assert!(group.is_member(&owner));
        assert!(group.is_member(&member));

        let stranger = UserId::new();
        assert!(!group.is_member(&stranger));
    }
}
