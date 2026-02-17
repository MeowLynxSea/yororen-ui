use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub category: TodoCategory,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TodoCategory {
    Work,
    Personal,
    Shopping,
    Health,
    Other,
}

impl TodoCategory {
    pub fn all() -> Vec<TodoCategory> {
        vec![
            TodoCategory::Work,
            TodoCategory::Personal,
            TodoCategory::Shopping,
            TodoCategory::Health,
            TodoCategory::Other,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TodoCategory::Work => "工作",
            TodoCategory::Personal => "个人",
            TodoCategory::Shopping => "购物",
            TodoCategory::Health => "健康",
            TodoCategory::Other => "其他",
        }
    }
}

impl Todo {
    pub fn new(title: String, category: TodoCategory) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
            category,
        }
    }
}
