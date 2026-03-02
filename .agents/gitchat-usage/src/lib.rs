use uuid::Uuid;

#[derive(Clone)]
pub struct Message<ChatType> {
    uuid:Uuid,
    content:ChatType
}

pub enum GcError<ChatType> {
    UuidNotFound,
    StringNotFound,
    MergeRecordNotFound,
    ThingExist,
    GcMergeHumanError(Breach<ChatType>, Breach<ChatType>)
}

#[derive(Clone)]
pub enum IsForked {
    False,
    True(Uuid, Uuid)
}

#[derive(Clone)]
struct Breach<ChatType> {
    messages: Vec<Message<ChatType>>,
    is_forked: IsForked,
    breach_id: Uuid,
    name: String
}

struct Root<ChatType> {
    breaches: Vec<Breach<ChatType>>,
    name: String
}

impl<ChatType: Clone> Breach<ChatType> {
    fn new(name: String) -> Self {
        Self {
            messages: vec![],
            is_forked: IsForked::False,
            breach_id: Uuid::new_v4(),
            name: name
        }
    }
    fn fork(&self, name: String, forked_on: I64OrUuid) -> Result<Self, GcError<ChatType>> {
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
            is_forked: IsForked::True(self.breach_id, MsgId),
            breach_id: Uuid::new_v4(),
            name: name
        })
    }
}

// 这个枚举的作用是让用户既可以传String, 也可以传uuid。这是一个备用设计。
enum StringOrUuid {
    Name(String),
    BreachId(Uuid)
}

// 同上
enum I64OrUuid {
    Index(i64),
    MessageId(Uuid)
}

impl<ChatType: Clone> Root<ChatType> {
    fn new(name: String) -> Self {
        Self {
            breaches: vec![],
            name: name
        }
    }
    fn create_breach(&mut self, name: String) -> Result<Uuid, GcError<ChatType>> {
        for i in &mut self.breaches {
            if i.name == name {
                return Err(GcError::ThingExist);
            }
        }
        &mut self.breaches.push(Breach::new(name.clone()));
        for i in &mut self.breaches {
            if i.name == name {
                return Ok(i.breach_id);
            }
        }
        unreachable!()
    }
    fn fork_breach(&mut self, forked_on: StringOrUuid, forked_on_message: I64OrUuid, name: String) -> Result<Uuid, GcError<ChatType>> {
        for i in &mut self.breaches {
            if i.name == name {
                return Err(GcError::ThingExist);
            }
        }
        match forked_on {
            StringOrUuid::Name(forked_on_name) => {
                for i in &mut self.breaches {
                    if i.name == forked_on_name {
                        let new_breach = i.fork(name.clone(), forked_on_message)?;
                        let new_id = new_breach.breach_id;
                        self.breaches.push(new_breach);
                        return Ok(new_id);
                    }
                }
                return Err(GcError::StringNotFound);
            } 
            StringOrUuid::BreachId(forked_on_uuid) => {
                for i in &mut self.breaches {
                    if i.breach_id == forked_on_uuid {
                        let new_breach = i.fork(name.clone(), forked_on_message)?;
                        let new_id = new_breach.breach_id;
                        self.breaches.push(new_breach);
                        return Ok(new_id);
                    }
                }
                return Err(GcError::UuidNotFound);
            }
        }
    }
} 

#[derive(PartialEq)]
enum MergeMode {
    Force,
    Human
}

pub enum ManualMergeAction<ChatType> {
    UseFrom,
    UseTo,
    UseMessages(Vec<Message<ChatType>>)
}

impl<ChatType: Clone> Root<ChatType> {
    fn find_breach_index_by_uuid(&self, id: &Uuid) -> Result<usize, GcError<ChatType>> {
        let mut index:usize = 0;
        for i in &self.breaches {
            if i.breach_id == *id {
                return Ok(index);
            }
            index = index + 1;
        }
        return Err(GcError::UuidNotFound);
    }
    fn remove_breach(&mut self, id: &Uuid) -> Result<(), GcError<ChatType>> {
        let index = self.find_breach_index_by_uuid(id)?;
        self.breaches.remove(index);
        Ok(())
    }
    fn merge_base(&mut self, from: Uuid, to: Uuid) -> Result<(), GcError<ChatType>> {
        let from_index = self.find_breach_index_by_uuid(&from)?;
        let to_index = self.find_breach_index_by_uuid(&to)?;

        // Merge direction is always `from -> to`.
        // Keep target identity (`to`) and clear fork metadata to avoid dangling refs.
        let mut merged = self.breaches[from_index].clone();
        merged.breach_id = self.breaches[to_index].breach_id;
        merged.name = self.breaches[to_index].name.clone();
        merged.is_forked = IsForked::False;

        self.breaches[to_index] = merged;
        Ok(())
    }
    fn merge_tool(&mut self, from: Uuid, to: Uuid, mode: MergeMode) -> Result<(), GcError<ChatType>> {
        if from == to {
            return Ok(());
        }

        let from_index = self.find_breach_index_by_uuid(&from)?;
        let to_index = self.find_breach_index_by_uuid(&to)?;
        let from_breach = self.breaches[from_index].clone();
        let to_breach = self.breaches[to_index].clone();

        if mode == MergeMode::Force {
            return self.merge_base(from, to);
        }

        let from_last = from_breach.messages.last().map(|m| m.uuid);
        let to_last = to_breach.messages.last().map(|m| m.uuid);

        // Case 1: `from` is forked from `to` (child -> parent)
        if let IsForked::True(parent_id, fork_line) = &from_breach.is_forked {
            if *parent_id == to_breach.breach_id {
                if Some(*fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                if Some(*fork_line) == from_last {
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_breach, to_breach));
            }
        }

        // Case 2: `to` is forked from `from` (parent -> child)
        if let IsForked::True(parent_id, fork_line) = &to_breach.is_forked {
            if *parent_id == from_breach.breach_id {
                if Some(*fork_line) == from_last {
                    return Ok(());
                }
                if Some(*fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_breach, to_breach));
            }
        }

        // Case 3: both are forked from the same parent (siblings)
        if let (IsForked::True(from_parent, from_fork_line), IsForked::True(to_parent, to_fork_line)) =
            (&from_breach.is_forked, &to_breach.is_forked)
        {
            if from_parent == to_parent {
                if Some(*from_fork_line) == from_last {
                    return Ok(());
                }
                if Some(*to_fork_line) == to_last {
                    self.merge_base(from, to)?;
                    return Ok(());
                }
                return Err(GcError::GcMergeHumanError(from_breach, to_breach));
            }
        }

        Err(GcError::MergeRecordNotFound)
    }

    fn merge_manual(
        &mut self,
        from: Uuid,
        to: Uuid,
        action: ManualMergeAction<ChatType>
    ) -> Result<(), GcError<ChatType>> {
        if from == to {
            return Ok(());
        }

        // Validate both branches exist before applying manual strategy.
        self.find_breach_index_by_uuid(&from)?;
        let to_index = self.find_breach_index_by_uuid(&to)?;

        match action {
            ManualMergeAction::UseFrom => self.merge_base(from, to),
            ManualMergeAction::UseTo => Ok(()),
            ManualMergeAction::UseMessages(messages) => {
                self.breaches[to_index].messages = messages;
                self.breaches[to_index].is_forked = IsForked::False;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests;
