use uuid::Uuid;

#[derive(Clone)]
pub struct Message<ChatType> {
    pub uuid:Uuid,
    pub content:ChatType
}

pub enum GcError<ChatType> {
    UuidNotFound,
    StringNotFound,
    MergeRecordNotFound,
    ThingExist,
    GcMergeHumanError(Branch<ChatType>, Branch<ChatType>)
}

#[derive(Clone)]
pub enum IsForked {
    False,
    True(Uuid, Uuid)
}

#[derive(Clone)]
pub struct Branch<ChatType> {
    pub messages: Vec<Message<ChatType>>,
    pub is_forked: IsForked,
    pub branch_id: Uuid,
    pub name: String
}

pub struct Root<ChatType> {
    pub branches: Vec<Branch<ChatType>>,
    pub name: String
}

impl<ChatType: Clone> Branch<ChatType> {
    pub fn new(name: String) -> Self {
        Self {
            messages: vec![],
            is_forked: IsForked::False,
            branch_id: Uuid::new_v4(),
            name: name
        }
    }
    pub fn fork(&self, name: String, forked_on: I64OrUuid) -> Result<Self, GcError<ChatType>> {
        let MsgId:Uuid;
        let mut finded_vector:Vec<Message<ChatType>>;
        match forked_on {
            I64OrUuid::MessageId(msg_id) => {
                let mut finded_id:bool = false;
                finded_vector = vec![];
                for i in &self.messages {
                    finded_vector.push(i.clone());
                    if i.uuid == msg_id {
                        finded_id = true;
                        break;
                    }
                }
                if !finded_id {
                    return Err(GcError::UuidNotFound);
                }
                MsgId = msg_id;
            }
            I64OrUuid::Index(MsgIndex) => {
                if ((&self.messages.len() - 1) as i64 )  < MsgIndex {
                    return Err(GcError::UuidNotFound);
                }
                let mut tmp_index = 0;
                finded_vector = vec![];
                for i in &self.messages {
                    if tmp_index <= MsgIndex {
                        finded_vector.push(i.clone());
                        tmp_index = tmp_index + 1;
                    } else {
                        break;
                    }
                }
                MsgId = finded_vector.last().unwrap().uuid;
            }
        }
        Ok(Self {
            messages: finded_vector,
            is_forked: IsForked::True(self.branch_id, MsgId),
            branch_id: Uuid::new_v4(),
            name: name
        })
    }
}

// 这个枚举的作用是让用户既可以传String, 也可以传uuid。这是一个备用设计。
pub enum StringOrUuid {
    Name(String),
    BranchId(Uuid)
}

// 同上
pub enum I64OrUuid {
    Index(i64),
    MessageId(Uuid)
}

impl<ChatType: Clone> Root<ChatType> {
    pub fn new(name: String) -> Self {
        Self {
            branches: vec![],
            name: name
        }
    }
    pub fn create_branch(&mut self, name: String) -> Result<Uuid, GcError<ChatType>> {
        for i in &mut self.branches {
            if i.name == name {
                return Err(GcError::ThingExist);
            }
        }
        &mut self.branches.push(Branch::new(name.clone()));
        for i in &mut self.branches {
            if i.name == name {
                return Ok(i.branch_id);
            }
        }
        unreachable!()
    }
    pub fn fork_branch(&mut self, forked_on: StringOrUuid, forked_on_message: I64OrUuid, name: String) -> Result<Uuid, GcError<ChatType>> {
        for i in &mut self.branches {
            if i.name == name {
                return Err(GcError::ThingExist);
            }
        }
        match forked_on {
            StringOrUuid::Name(forked_on_name) => {
                for i in &mut self.branches {
                    if i.name == forked_on_name {
                        let new_branch = i.fork(name.clone(), forked_on_message)?;
                        let new_id = new_branch.branch_id;
                        self.branches.push(new_branch);
                        return Ok(new_id);
                    }
                }
                return Err(GcError::StringNotFound);
            } 
            StringOrUuid::BranchId(forked_on_uuid) => {
                for i in &mut self.branches {
                    if i.branch_id == forked_on_uuid {
                        let new_branch = i.fork(name.clone(), forked_on_message)?;
                        let new_id = new_branch.branch_id;
                        self.branches.push(new_branch);
                        return Ok(new_id);
                    }
                }
                return Err(GcError::UuidNotFound);
            }
        }
    }
} 

#[derive(PartialEq)]
pub enum MergeMode {
    Force,
    Human
}

pub enum ManualMergeAction<ChatType> {
    UseFrom,
    UseTo,
    UseMessages(Vec<Message<ChatType>>)
}

impl<ChatType: Clone> Root<ChatType> {
    pub fn find_branch_index_by_uuid(&self, id: &Uuid) -> Result<usize, GcError<ChatType>> {
        let mut index:usize = 0;
        for i in &self.branches {
            if i.branch_id == *id {
                return Ok(index);
            }
            index = index + 1;
        }
        return Err(GcError::UuidNotFound);
    }
    pub fn remove_branch(&mut self, id: &Uuid) -> Result<(), GcError<ChatType>> {
        let index = self.find_branch_index_by_uuid(id)?;
        self.branches.remove(index);
        Ok(())
    }
    pub fn merge_base(&mut self, from: Uuid, to: Uuid) -> Result<(), GcError<ChatType>> {
        let from_index = self.find_branch_index_by_uuid(&from)?;
        let to_index = self.find_branch_index_by_uuid(&to)?;

        // Merge direction is always `from -> to`.
        // Keep target identity (`to`) and clear fork metadata to avoid dangling refs.
        let mut merged = self.branches[from_index].clone();
        merged.branch_id = self.branches[to_index].branch_id;
        merged.name = self.branches[to_index].name.clone();
        merged.is_forked = IsForked::False;

        self.branches[to_index] = merged;
        Ok(())
    }
    pub fn merge_tool(&mut self, from: Uuid, to: Uuid, mode: MergeMode) -> Result<(), GcError<ChatType>> {
        if from == to {
            return Ok(());
        }

        let from_index = self.find_branch_index_by_uuid(&from)?;
        let to_index = self.find_branch_index_by_uuid(&to)?;
        let from_branch = self.branches[from_index].clone();
        let to_branch = self.branches[to_index].clone();

        if mode == MergeMode::Force {
            return self.merge_base(from, to);
        }

        let from_last = from_branch.messages.last().map(|m| m.uuid);
        let to_last = to_branch.messages.last().map(|m| m.uuid);

        // Case 1: `from` is forked from `to` (child -> parent)
        if let IsForked::True(parent_id, fork_line) = &from_branch.is_forked {
            if *parent_id == to_branch.branch_id {
                if Some(*fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                if Some(*fork_line) == from_last {
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_branch, to_branch));
            }
        }

        // Case 2: `to` is forked from `from` (parent -> child)
        if let IsForked::True(parent_id, fork_line) = &to_branch.is_forked {
            if *parent_id == from_branch.branch_id {
                if Some(*fork_line) == from_last {
                    return Ok(());
                }
                if Some(*fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_branch, to_branch));
            }
        }

        // Case 3: both are forked from the same parent (siblings)
        if let (IsForked::True(from_parent, from_fork_line), IsForked::True(to_parent, to_fork_line)) =
            (&from_branch.is_forked, &to_branch.is_forked)
        {
            if from_parent == to_parent {
                if Some(*from_fork_line) == from_last {
                    return Ok(());
                }
                if Some(*to_fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_branch, to_branch));
            }
        }

        Err(GcError::MergeRecordNotFound)
    }

    pub fn merge_manual(
        &mut self,
        from: Uuid,
        to: Uuid,
        action: ManualMergeAction<ChatType>
    ) -> Result<(), GcError<ChatType>> {
        if from == to {
            return Ok(());
        }

        // Validate both branches exist before applying manual strategy.
        self.find_branch_index_by_uuid(&from)?;
        let to_index = self.find_branch_index_by_uuid(&to)?;

        match action {
            ManualMergeAction::UseFrom => self.merge_base(from, to),
            ManualMergeAction::UseTo => Ok(()),
            ManualMergeAction::UseMessages(messages) => {
                self.branches[to_index].messages = messages;
                self.branches[to_index].is_forked = IsForked::False;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests;
